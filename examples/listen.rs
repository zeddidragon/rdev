use rdev::{listen, Event};

fn main() {
    // This will block.
    std::env::set_var("KEYBOARD_ONLY", "y");
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
}
