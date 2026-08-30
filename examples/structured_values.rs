//! Read and edit the pieces of a structured value, one at a time.
//!
//! A `GEO` is two `;`-separated components, a `CATEGORIES` and an `RDATE` are
//! `,`-separated lists, and a `REQUEST-STATUS` is both at once. The cursor
//! addresses either level explicitly: a whole-value read never truncates at a
//! separator, and a component read always says which slot it wants.
//!
//! Run with: `cargo run --example structured_values`

use ical::{
    component::vevent::VEVENT,
    prop::{categories::CATEGORIES, geo::GEO, rdate::RDATE, request_status::REQUEST_STATUS},
    tree::cst::IcalCst,
};

fn main() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:offsite@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000Z\r\n",
        "GEO:37.386013;-122.082932\r\n",
        "CATEGORIES:work,offsite,travel\r\n",
        "RDATE:20260112T090000Z,20260119T090000Z\r\n",
        "REQUEST-STATUS:2.0;Success\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let mut cst = IcalCst::parse(raw).unwrap();
    let event = cst.component_mut::<VEVENT>().expect("the event");

    // A `;`-structured value: each component read by its own index.
    {
        let geo = event.prop_mut::<GEO>().expect("a position");

        println!("GEO latitude  {:?}", geo.component(0));
        println!("GEO longitude {:?}", geo.component(1));

        // The whole value keeps its separator, since nothing asked for a slot.
        println!("GEO whole     {:?}", geo.text());
    }

    // A `,`-structured value: a list, read and written as one.
    {
        let mut categories = event.prop_mut::<CATEGORIES>().expect("the categories");

        println!("\nCATEGORIES {:?}", categories.list());

        categories.set_list(&["work", "offsite", "travel", "quarterly"]);
        println!("after adding one: {:?}", categories.list());
    }

    // An `RDATE` is a list too, of dates rather than text.
    {
        let dates = event.prop_mut::<RDATE>().expect("the extra dates");

        println!("\nRDATE {:?}", dates.list());
    }

    // A `REQUEST-STATUS` is `;`-structured, and each component may itself be a
    // list, so a read has to say which level it means.
    {
        let status = event.prop_mut::<REQUEST_STATUS>().expect("a status");

        println!("\nREQUEST-STATUS code        {:?}", status.component(0));
        println!("REQUEST-STATUS description {:?}", status.component(1));
    }

    // Only the line that was edited changed. Every other byte, folds included,
    // is the one it arrived as.
    print!("\n{cst}");
}
