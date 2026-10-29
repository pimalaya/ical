// The ical.js oracle.
//
// Reads the window list (a calendar name, a window start and a window end, tab
// separated) on stdin, and writes `name<TAB>uid<TAB>start<TAB>end` for the
// earliest TAKE occurrences of each series starting in the window, spelled as
// the Rust side spells them. A calendar the library refuses is one
// `name<TAB>ERROR<TAB>reason` line.
//
//     node oracle_icaljs.mjs ICALJS CORPUS ZONES <windows.tsv
//
// ICALJS is the unpacked ical.js package, CORPUS is tests/corpus: a calendar
// name with a slash is a path under it, a bare one is under its rie folder.
// ZONES is the zones.json of @nextcloud/timezones: the IANA VTIMEZONEs, and the
// Windows and legacy aliases, that Thunderbird generates and Nextcloud ships.
//
// The answer is ical.js's own: ICAL.parse and ICAL.Component read the
// calendar, its VTIMEZONEs are registered with ICAL.TimezoneService (ical.js
// also finds them in the component tree itself), and each series is an
// ICAL.Event with its overrides related, walked by its iterator and placed by
// getOccurrenceDetails. ical.js reads every component as a VEVENT, so what it
// has no notion of is filled in as RFC 5545 has it, the way the two other
// sides do: a VTODO starts at its DUE when it has no DTSTART and ends at its
// DUE, and a VTODO or VJOURNAL with neither DUE nor DURATION ends at its start.
//
// A series is every component of one name (VEVENT, VTODO or VJOURNAL) and one
// UID: the one with no RECURRENCE-ID and the highest SEQUENCE (the first
// listed of equals) is walked, every other one with a RECURRENCE-ID is related
// to it, in increasing SEQUENCE so the highest wins. An override with no series
// to relate to is the one occurrence it states. The components with no UID are
// each their own series, all under the label `-`.
//
// ical.js has no time zone database: its consumers register one. As they do,
// a TZID the calendar names and does not define is registered from ZONES,
// through its alias when it is one. The calendar's own VTIMEZONE always wins,
// and a TZID ZONES does not know stays unresolved, which ical.js reads as
// floating.
//
// Each calendar runs in its own worker, terminated past BUDGET_MS, since a
// rule ical.js cannot fulfil may never return to this script.

import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";

const TAKE = 200;
const MARGIN = 3 * 86400;
const BUDGET_MS = 20000;
const NAMES = ["vevent", "vtodo", "vjournal"];

if (isMainThread) {
  await main();
} else {
  parentPort.postMessage(await answer(workerData));
}

async function main() {
  const [icaljs, corpus, zones] = process.argv.slice(2);
  const input = readFileSync(0, "utf8");

  for (const line of input.split("\n")) {
    if (line === "") {
      continue;
    }

    const [name, lo, hi] = line.split("\t");
    const path = name.includes("/") ? resolve(corpus, name) : join(corpus, "rie", name);
    const result = await run({ icaljs, zones, path, lo, hi });

    if (result.error !== undefined) {
      process.stdout.write(`${name}\tERROR\t${result.error}\n`);
      continue;
    }

    for (const [uid, rows] of result.series) {
      for (const row of rows) {
        process.stdout.write(`${name}\t${uid}\t${row}\n`);
      }
    }
  }
}

function run(data) {
  return new Promise((resolve) => {
    const worker = new Worker(new URL(import.meta.url), { workerData: data });

    const timer = setTimeout(() => {
      worker.terminate();
      resolve({ error: "TIMEOUT" });
    }, BUDGET_MS);

    worker.once("message", (result) => {
      clearTimeout(timer);
      worker.terminate();
      resolve(result);
    });

    worker.once("error", (error) => {
      clearTimeout(timer);
      resolve({ error: reason(error) });
    });
  });
}

function reason(error) {
  const name = error?.constructor?.name ?? "Error";
  const message = String(error?.message ?? error).replace(/\s+/g, " ").slice(0, 80);
  return `${name}: ${message}`;
}

async function answer({ icaljs, zones, path, lo, hi }) {
  const ICAL = (await import(pathToFileURL(join(icaljs, "dist/ical.js")).href)).default;
  const database = JSON.parse(readFileSync(zones, "utf8"));

  try {
    const text = readFileSync(path, "utf8");
    return { series: walk(ICAL, database, text, day(lo), day(hi)) };
  } catch (error) {
    return { error: reason(error) };
  }
}

// A window day (YYYYMMDD) as seconds since the epoch, on UTC.
function day(text) {
  const at = new Date(0);
  at.setUTCFullYear(Number(text.slice(0, 4)), Number(text.slice(4, 6)) - 1, Number(text.slice(6, 8)));
  return at.getTime() / 1000;
}

function walk(ICAL, database, text, lo, hi) {
  // NOTE: Every Item below is defined against this ICAL, loaded in this
  // worker only.
  class Item extends ICAL.Event {
    get startDate() {
      const start = this._firstProp("dtstart");

      if (start || this.component.name !== "vtodo") {
        return start;
      }

      return this._firstProp("due");
    }

    get endDate() {
      const name = this.component.name;

      if (name === "vevent") {
        return super.endDate;
      }

      const due = name === "vtodo" ? this._firstProp("due") : null;

      if (due) {
        return due;
      }

      if (this._firstProp("duration")) {
        return super.endDate;
      }

      return this.startDate.clone();
    }
  }

  // NOTE: Lenient, as Thunderbird and Nextcloud run it: a DATE-TIME property
  // written as a date (`DTSTART:20180110`) is read as the date it is rather
  // than refused.
  ICAL.design.strict = false;

  let jcal = ICAL.parse(text);

  // NOTE: One root component parses to its jCal, several (an iTIP exchange)
  // to a list of them, of which the first is read, as the ical-rs side reads
  // it.
  if (typeof jcal[0] !== "string") {
    jcal = jcal[0];
  }

  const roots = [new ICAL.Component(jcal)];

  ICAL.TimezoneService.reset();

  for (const root of roots) {
    for (const zone of root.getAllSubcomponents("vtimezone")) {
      try {
        ICAL.TimezoneService.register(zone);
      } catch {
        // A VTIMEZONE ical.js cannot read is no zone of its.
      }
    }
  }

  for (const root of roots) {
    const defined = new Set(
      root.getAllSubcomponents("vtimezone").map((zone) => zone.getFirstPropertyValue("tzid")),
    );

    for (const tzid of named(root)) {
      if (defined.has(tzid) || ICAL.TimezoneService.has(tzid)) {
        continue;
      }

      const zone = lookup(ICAL, database, tzid);

      if (zone) {
        ICAL.TimezoneService.register(zone, tzid);
      }
    }
  }

  const components = roots.flatMap((root) =>
    root.getAllSubcomponents().filter((component) => NAMES.includes(component.name)),
  );

  const groups = new Map();
  const series = new Map();

  const push = (uid, rows) => {
    if (!series.has(uid)) {
      series.set(uid, []);
    }

    series.get(uid).push(...rows);
  };

  for (const component of components) {
    const uid = component.getFirstPropertyValue("uid");

    if (uid === null || uid === undefined) {
      push("-", occurrences(ICAL, new Item(component, { exceptions: [] }), [], lo, hi));
      continue;
    }

    const key = `${component.name}\u0000${uid}`;

    if (!groups.has(key)) {
      groups.set(key, { uid: String(uid), masters: [], overrides: [] });
    }

    const group = groups.get(key);
    const override = component.hasProperty("recurrence-id");
    (override ? group.overrides : group.masters).push(component);
  }

  for (const { uid, masters, overrides } of groups.values()) {
    const sequence = (component) => Number(component.getFirstPropertyValue("sequence") ?? 0) || 0;
    const exceptions = overrides
      .map((component, index) => ({ component, index }))
      .sort((a, b) => sequence(a.component) - sequence(b.component) || a.index - b.index)
      .map(({ component }) => new Item(component, { exceptions: [] }));

    let master = null;

    for (const component of masters) {
      if (master === null || sequence(component) > sequence(master)) {
        master = component;
      }
    }

    if (master === null) {
      for (const exception of exceptions) {
        push(uid, single(ICAL, exception, lo, hi));
      }

      continue;
    }

    push(uid, occurrences(ICAL, new Item(master, { exceptions }), exceptions, lo, hi));
  }

  return [...series.keys()]
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
    .map((uid) => {
      const rows = series
        .get(uid)
        .sort((a, b) => a[0] - b[0] || (a[1] < b[1] ? -1 : a[1] > b[1] ? 1 : 0))
        .slice(0, TAKE)
        .map(([, row]) => row);
      return [uid, rows];
    })
    .filter(([, rows]) => rows.length > 0);
}

// Every TZID a component of the tree names, outside its VTIMEZONEs.
function named(component, tzids = new Set()) {
  if (component.name === "vtimezone") {
    return tzids;
  }

  for (const prop of component.getAllProperties()) {
    const tzid = prop.getParameter("tzid");

    if (typeof tzid === "string") {
      tzids.add(tzid);
    }
  }

  for (const child of component.getAllSubcomponents()) {
    named(child, tzids);
  }

  return tzids;
}

// The zone the database gives a TZID, through its aliases, as Nextcloud's
// TimezoneManager builds it: the stored observances in a VTIMEZONE of that
// TZID.
function lookup(ICAL, database, tzid) {
  let name = tzid;

  for (let level = 0; level < 20; level++) {
    if (name === "UTC") {
      return ICAL.Timezone.utcTimezone;
    }

    const zone = database.zones[name];

    if (zone) {
      const ics = ["BEGIN:VTIMEZONE", `TZID:${name}`, ...zone.ics, "END:VTIMEZONE"];
      return new ICAL.Timezone(new ICAL.Component(ICAL.parse(ics.join("\r\n"))));
    }

    name = database.aliases[name];

    if (name === undefined) {
      return null;
    }
  }

  return null;
}

// The occurrences of a series starting in the window, as `[key, row]`.
function occurrences(ICAL, event, exceptions, lo, hi) {
  if (!event.startDate) {
    return [];
  }

  // NOTE: The iterator yields identities in order, an override moving its
  // occurrence by as much as its own shift, so the walk goes that far past the
  // window.
  let shift = 0;

  for (const exception of exceptions) {
    if (exception.startDate) {
      shift = Math.max(shift, key(ICAL, exception.recurrenceId) - key(ICAL, exception.startDate));
    }
  }

  const until = hi + MARGIN + shift;
  const iterator = event.iterator();
  const rows = [];
  let next;

  while ((next = iterator.next())) {
    if (key(ICAL, next) >= until) {
      break;
    }

    const details = event.getOccurrenceDetails(next);
    const row = place(ICAL, details.startDate, details.endDate, lo, hi);

    if (row) {
      rows.push(row);
    }
  }

  return rows;
}

// The one occurrence an override with no series states.
function single(ICAL, exception, lo, hi) {
  if (!exception.startDate) {
    return [];
  }

  const row = place(ICAL, exception.startDate, exception.endDate, lo, hi);
  return row ? [row] : [];
}

function place(ICAL, start, end, lo, hi) {
  const at = key(ICAL, start);

  if (at < lo || at >= hi) {
    return null;
  }

  return [at, `${spell(ICAL, start)}\t${spell(ICAL, end ?? start)}`];
}

function floating(ICAL, time) {
  return time.isDate || !time.zone || time.zone === ICAL.Timezone.localTimezone || time.zone.tzid === "floating";
}

// A time as seconds on UTC, a floating one and a date as if on UTC, which is
// how the Rust side orders and windows them.
function key(ICAL, time) {
  const at = floating(ICAL, time) ? time : time.convertToZone(ICAL.Timezone.utcTimezone);
  const date = new Date(0);
  date.setUTCFullYear(at.year, at.month - 1, at.day);
  date.setUTCHours(at.isDate ? 0 : at.hour, at.isDate ? 0 : at.minute, at.isDate ? 0 : at.second);
  return date.getTime() / 1000;
}

function spell(ICAL, time) {
  const pad = (value, width) => String(value).padStart(width, "0");
  const at = floating(ICAL, time) ? time : time.convertToZone(ICAL.Timezone.utcTimezone);
  const day = `${pad(at.year, 4)}${pad(at.month, 2)}${pad(at.day, 2)}`;

  if (time.isDate) {
    return day;
  }

  const clock = `T${pad(at.hour, 2)}${pad(at.minute, 2)}${pad(at.second, 2)}`;
  return floating(ICAL, time) ? day + clock : `${day}${clock}Z`;
}
