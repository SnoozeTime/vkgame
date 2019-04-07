use std::fs;

fn main() {
    for r in fs::read_dir("assets/shaders/").unwrap() {
        println!("{:?}", r.unwrap().path());
    }

    let p = std::path::Path::new("assets");
    dbg!(p.join("hi"));
}
