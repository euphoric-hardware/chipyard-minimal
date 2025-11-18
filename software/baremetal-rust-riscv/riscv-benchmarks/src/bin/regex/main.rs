#![no_main]
#![no_std]

extern crate alloc;

use core::fmt::Write;
use regex_automata::meta::Regex;
use riscv_benchmarks::*;
use riscv_rt::entry;

// Test data containing various patterns to match
const TEST_TEXT: &str = "\
Email addresses: alice@example.com, bob123@test.org, charlie_xyz@company.co.uk
Phone numbers: 555-1234, (123) 456-7890, 800.555.1212
Dates: 2024-01-15, 12/25/2023, 2023.11.17
URLs: https://www.rust-lang.org, http://github.com/rust-lang
IP addresses: 192.168.1.1, 10.0.0.255, 172.16.254.1
Numbers: 42, 3.14159, -273.15, 1,234,567
Words: The quick brown fox jumps over the lazy dog
";

// Additional text for more intensive matching
const LONG_TEXT: &str = "\
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer 123 nec odio.
Praesent libero. Sed cursus ante dapibus diam@email.com. Sed nisi. Nulla quis
sem at nibh elementum imperdiet 456.789. Duis sagittis ipsum. Praesent mauris.
Fusce nec tellus sed augue semper porta. Mauris massa. Vestibulum lacinia arcu
eget nulla. Class aptent taciti sociosqu ad litora torquent per conubia nostra,
per inceptos himenaeos 2024-03-15. Curabitur sodales ligula in libero.
test@domain.org and another.email@example.com with 192.168.0.1 addresses.
Phone: 555-0123 or (800) 123-4567. Visit https://example.com for more info.
";

fn benchmark_email_extraction() -> usize {
    let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    let mut count = 0;

    for _ in 0..100 {
        for _ in re.find_iter(TEST_TEXT) {
            count += 1;
        }
        for _ in re.find_iter(LONG_TEXT) {
            count += 1;
        }
    }
    count
}

fn benchmark_phone_extraction() -> usize {
    let re = Regex::new(r"[0-9]{3}[-.]?[0-9]{3}[-.]?[0-9]{4}|\([0-9]{3}\)[[:space:]]*[0-9]{3}[-.]?[0-9]{4}").unwrap();

    let mut count = 0;

    for _ in 0..100 {
        for _ in re.find_iter(TEST_TEXT) {
            count += 1;
        }
        for _ in re.find_iter(LONG_TEXT) {
            count += 1;
        }
    }

    count
}

fn benchmark_date_extraction() -> usize {
    let re = Regex::new(
        r"[0-9]{4}[-./][0-9]{2}[-./][0-9]{2}|[0-9]{2}[-./][0-9]{2}[-./][0-9]{4}"
    ).unwrap();
    let mut count = 0;

    for _ in 0..100 {
        for _ in re.find_iter(TEST_TEXT) {
            count += 1;
        }
        for _ in re.find_iter(LONG_TEXT) {
            count += 1;
        }
    }

    count
}

fn benchmark_ip_extraction() -> usize {
    let re = Regex::new(
        r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}"
    ).unwrap();
    let mut count = 0;

    for _ in 0..100 {
        for _ in re.find_iter(TEST_TEXT) {
            count += 1;
        }
        for _ in re.find_iter(LONG_TEXT) {
            count += 1;
        }
    }

    count
}

fn benchmark_url_extraction() -> usize {
    let re = Regex::new(r"https?://[a-zA-Z0-9./\-]+").unwrap();
    let mut count = 0;

    for _ in 0..100 {
        for _ in re.find_iter(TEST_TEXT) {
            count += 1;
        }
        for _ in re.find_iter(LONG_TEXT) {
            count += 1;
        }
    }

    count
}

fn benchmark_word_boundaries() -> usize {
    let re = Regex::new(
        r"[A-Za-z]{4,}"
    ).unwrap();
    let mut count = 0;

    for _ in 0..50 {
        for _ in re.find_iter(TEST_TEXT) {
            count += 1;
        }
        for _ in re.find_iter(LONG_TEXT) {
            count += 1;
        }
    }

    count
}

fn benchmark_number_extraction() -> usize {
    let re = Regex::new(
        r"-?[0-9]+\.?[0-9]*"
    ).unwrap();

    let mut count = 0;

    for _ in 0..100 {
        for _ in re.find_iter(TEST_TEXT) {
            count += 1;
        }
        for _ in re.find_iter(LONG_TEXT) {
            count += 1;
        }
    }

    count
}

#[entry]
fn main() -> ! {
    init_heap();

    writeln!(htif::HostFile::stdout(), "Starting regex benchmarks...").unwrap();

    writeln!(htif::HostFile::stdout(), "\nEmail extraction").unwrap();
    let benchmark_data = start_benchmark();
    let email_count = benchmark_email_extraction();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "   Matches found: {}", email_count).unwrap();

    writeln!(htif::HostFile::stdout(), "\nPhone number extraction").unwrap();
    let benchmark_data = start_benchmark();
    let phone_count = benchmark_phone_extraction();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "   Matches found: {}", phone_count).unwrap();

    writeln!(htif::HostFile::stdout(), "\nDate extraction").unwrap();
    let benchmark_data = start_benchmark();
    let date_count = benchmark_date_extraction();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "   Matches found: {}", date_count).unwrap();

    writeln!(htif::HostFile::stdout(), "\nIP address extraction").unwrap();
    let benchmark_data = start_benchmark();
    let ip_count = benchmark_ip_extraction();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "   Matches found: {}", ip_count).unwrap();

    writeln!(htif::HostFile::stdout(), "\nURL extraction").unwrap();
    let benchmark_data = start_benchmark();
    let url_count = benchmark_url_extraction();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "   Matches found: {}", url_count).unwrap();

    writeln!(htif::HostFile::stdout(), "\nWord boundary matching").unwrap();
    let benchmark_data = start_benchmark();
    let word_count = benchmark_word_boundaries();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "   Matches found: {}", word_count).unwrap();

    writeln!(htif::HostFile::stdout(), "\nNumber extraction").unwrap();
    let benchmark_data = start_benchmark();
    let num_count = benchmark_number_extraction();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "   Matches found: {}", num_count).unwrap();

    writeln!(htif::HostFile::stdout(), "\nAll regex benchmarks completed!").unwrap();

    exit();
}
