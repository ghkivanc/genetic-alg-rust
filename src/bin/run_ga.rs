use Genetic_Alg::*;  // Replace with your actual crate name

fn main() {
    let mut test_run = Run::new(0.322, 0.00522, 10, 30, 2);
    let result = test_run.run(1000);
    
    match save_iter_to_csv(&result.1, "run_3.csv") {
       Ok(_) => println!("Successfully wrote to CSV in current directory"),
       Err(e) => println!("Error: {}", e)
    }
}
