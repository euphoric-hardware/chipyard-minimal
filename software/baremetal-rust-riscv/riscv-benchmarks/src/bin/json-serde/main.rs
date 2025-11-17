#![no_main]
#![no_std]

extern crate alloc;

use riscv_benchmarks::*;
use riscv_rt::entry;
use serde::{Deserialize, Serialize};
use core::fmt::Write;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
    name: heapless::String<32>,
    age: u32,
    email: heapless::String<64>,
    active: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Company {
    name: heapless::String<32>,
    employees: heapless::Vec<Person, 10>,
    revenue: u64,
}

fn create_test_data() -> Company {
    let mut employees = heapless::Vec::new();

    let _ = employees.push(Person {
        name: heapless::String::try_from("Alice Johnson").unwrap(),
        age: 30,
        email: heapless::String::try_from("alice@example.com").unwrap(),
        active: true,
    });

    let _ = employees.push(Person {
        name: heapless::String::try_from("Bob Smith").unwrap(),
        age: 45,
        email: heapless::String::try_from("bob@example.com").unwrap(),
        active: true,
    });

    let _ = employees.push(Person {
        name: heapless::String::try_from("Carol White").unwrap(),
        age: 28,
        email: heapless::String::try_from("carol@example.com").unwrap(),
        active: false,
    });

    let _ = employees.push(Person {
        name: heapless::String::try_from("David Brown").unwrap(),
        age: 52,
        email: heapless::String::try_from("david@example.com").unwrap(),
        active: true,
    });

    let _ = employees.push(Person {
        name: heapless::String::try_from("Eve Davis").unwrap(),
        age: 35,
        email: heapless::String::try_from("eve@example.com").unwrap(),
        active: true,
    });

    Company {
        name: heapless::String::try_from("TechCorp").unwrap(),
        employees,
        revenue: 1_000_000_000,
    }
}

#[entry]
fn main() -> ! {
    init_heap();

    let company = create_test_data();
    let benchmark_data = start_benchmark();

    // Serialize to JSON
    let mut buffer = [0u8; 1024];
    let serialized_len = serde_json_core::to_slice(&company, &mut buffer)
        .expect("Serialization failed");

    let serialized = &buffer[..serialized_len];

    // Deserialize back
    let (deserialized, _): (Company, _) = serde_json_core::from_slice(serialized)
        .expect("Deserialization failed");

    print_benchmark_data(benchmark_data);

    // Verify round-trip
    if company != deserialized {
        panic!("Round-trip serialization failed!");
    }

    writeln!(htif::HostFile::stdout(), "JSON size: {} bytes", serialized.len()).unwrap();
    writeln!(htif::HostFile::stdout(), "Employees: {}", deserialized.employees.len()).unwrap();

    exit();
}
