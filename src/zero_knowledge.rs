pub struct Person;

impl Person {
    pub fn new() -> Self {
        Person
    }
}

impl KnowsSecretKey for Person {
    fn run(&self) {
        println!("Person is running");
    }
}
pub async fn run_simulation() {
    
}