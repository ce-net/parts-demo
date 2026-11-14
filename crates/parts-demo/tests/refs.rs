//! Every value this cell shows carries its source (ce-ref, ce-load README): the Status reply.
use parts_schema::Part;

#[test]
fn every_shown_value_has_a_ref() {
    let s = ce_db_dev::serve("127.0.0.1:0", ce_db_dev::Grants::all_of("rw", "parts-demo")).unwrap();
    let parts = ce_db::open_at("parts-demo", &s.addr.to_string(), "rw").unwrap().collection::<Part>().unwrap();
    let c = parts_demo::Cell::new(parts);
    ce_ref::assert_sourced(&c.status().unwrap());
}
