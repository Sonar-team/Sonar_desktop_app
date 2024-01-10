#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

// #[test]
// fn test_run_app_with_options() {
//     use std::process::Command;
//     // Run the application with custom arguments
//     let output = Command::new("target/debug/sonar_cli_app")
//         .arg("--output=my_output.csv")
//         .arg("--interface=lo")
//         .arg("--time=1")
//         .output()
//         .expect("Failed to run command");

//     assert!(output.status.success()); // Check for successful exit
//                                       // Add more checks on `output.stdout` or `output.stderr` if you want
// }

// #[test]
// fn test_scan_until_interrupt_integration() {
//     use std::sync::Arc;
//     use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
//     use std::thread;
//     use std::time::Duration;

//     let running = Arc::new(AtomicBool::new(true));
//     let r = running.clone();

//     // Assume "output.csv" and "all" are the parameters you would use for the scan_until_interrupt function
//     thread::spawn(move || {
//         // Call the actual function here.
//         sonar_cli_app::scan_until_interrupt("output.csv", "all");
//     });

//     // Wait a moment before simulating the Ctrl+C event
//     thread::sleep(Duration::from_secs(1));

//     // Simulate Ctrl+C by manually triggering the event handler
//     ctrlc::set_handler(move || {
//         r.store(false, SeqCst);
//         // Add any additional cleanup code here
//     }).expect("Error setting Ctrl-C handler");

//     // Wait for the function to clean up and exit
//     thread::sleep(Duration::from_secs(1));

//     // Validate that running has been set to false, indicating a graceful shutdown
//     assert_eq!(running.load(SeqCst), false);

//     // Add more validations such as checking if a CSV file has been created, etc.
// }
