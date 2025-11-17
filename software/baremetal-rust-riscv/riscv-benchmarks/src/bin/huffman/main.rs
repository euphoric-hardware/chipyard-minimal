#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BinaryHeap;
use riscv_benchmarks::*;
use riscv_rt::entry;
use core::fmt::Write;
use core::cmp::Ordering;

// Huffman tree node
#[derive(Debug, Clone)]
enum HuffmanNode {
    Leaf {
        ch: u8,
        freq: usize,
    },
    Internal {
        freq: usize,
        left: usize,
        right: usize,
    },
}

impl HuffmanNode {
    fn freq(&self) -> usize {
        match self {
            HuffmanNode::Leaf { freq, .. } => *freq,
            HuffmanNode::Internal { freq, .. } => *freq,
        }
    }
}

// Wrapper for priority queue (min-heap by frequency)
#[derive(Debug, Clone)]
struct HeapNode {
    index: usize,
    freq: usize,
}

impl Eq for HeapNode {}

impl PartialEq for HeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq) // Reverse for min-heap
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct HuffmanCoder {
    nodes: Vec<HuffmanNode>,
    root: usize,
}

impl HuffmanCoder {
    // Build Huffman tree from frequency map
    fn build(text: &[u8]) -> Self {
        let mut freq_map = [0usize; 256];
        for &byte in text {
            freq_map[byte as usize] += 1;
        }

        let mut nodes = Vec::new();
        let mut heap = BinaryHeap::new();

        // Create leaf nodes
        for (ch, &freq) in freq_map.iter().enumerate() {
            if freq > 0 {
                let index = nodes.len();
                nodes.push(HuffmanNode::Leaf {
                    ch: ch as u8,
                    freq,
                });
                heap.push(HeapNode { index, freq });
            }
        }

        // Build tree
        while heap.len() > 1 {
            let left_node = heap.pop().unwrap();
            let right_node = heap.pop().unwrap();

            let combined_freq = left_node.freq + right_node.freq;
            let index = nodes.len();
            nodes.push(HuffmanNode::Internal {
                freq: combined_freq,
                left: left_node.index,
                right: right_node.index,
            });
            heap.push(HeapNode {
                index,
                freq: combined_freq,
            });
        }

        let root = heap.pop().unwrap().index;

        HuffmanCoder { nodes, root }
    }

    // Generate codes for each character
    fn generate_codes(&self) -> [Option<Vec<bool>>; 256] {
        let mut codes: [Option<Vec<bool>>; 256] = [const { None }; 256];
        let mut current_code = Vec::new();
        self.generate_codes_helper(self.root, &mut current_code, &mut codes);
        codes
    }

    fn generate_codes_helper(
        &self,
        node_idx: usize,
        current_code: &mut Vec<bool>,
        codes: &mut [Option<Vec<bool>>; 256],
    ) {
        match &self.nodes[node_idx] {
            HuffmanNode::Leaf { ch, .. } => {
                codes[*ch as usize] = Some(current_code.clone());
            }
            HuffmanNode::Internal { left, right, .. } => {
                current_code.push(false);
                self.generate_codes_helper(*left, current_code, codes);
                current_code.pop();

                current_code.push(true);
                self.generate_codes_helper(*right, current_code, codes);
                current_code.pop();
            }
        }
    }

    // Encode text
    fn encode(&self, text: &[u8]) -> Vec<bool> {
        let codes = self.generate_codes();
        let mut result = Vec::new();

        for &ch in text {
            if let Some(code) = &codes[ch as usize] {
                result.extend_from_slice(code);
            }
        }

        result
    }

    // Decode bit sequence
    fn decode(&self, bits: &[bool]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut current_node = self.root;

        for &bit in bits {
            match &self.nodes[current_node] {
                HuffmanNode::Leaf { ch, .. } => {
                    result.push(*ch);
                    current_node = self.root;

                    // Process the current bit with the new node
                    match &self.nodes[current_node] {
                        HuffmanNode::Internal { left, right, .. } => {
                            current_node = if bit { *right } else { *left };
                        }
                        _ => {}
                    }
                }
                HuffmanNode::Internal { left, right, .. } => {
                    current_node = if bit { *right } else { *left };
                }
            }
        }

        // Handle last character
        if let HuffmanNode::Leaf { ch, .. } = &self.nodes[current_node] {
            result.push(*ch);
        }

        result
    }
}

const TEST_TEXT: &[u8] = b"The quick brown fox jumps over the lazy dog. \
                           This is a test of Huffman encoding and decoding. \
                           Huffman coding is a lossless data compression algorithm.";

#[entry]
fn main() -> ! {
    init_heap();

    writeln!(htif::HostFile::stdout(), "=== Huffman Encoding Benchmark ===").unwrap();
    writeln!(htif::HostFile::stdout(), "Original size: {} bytes", TEST_TEXT.len()).unwrap();

    // Build Huffman tree
    let build_benchmark = start_benchmark();
    let coder = HuffmanCoder::build(TEST_TEXT);
    print_benchmark_data(build_benchmark);

    // Encode
    let encode_benchmark = start_benchmark();
    let encoded = coder.encode(TEST_TEXT);
    print_benchmark_data(encode_benchmark);

    let encoded_bytes = (encoded.len() + 7) / 8;
    let compression_ratio = (TEST_TEXT.len() as f32) / (encoded_bytes as f32);
    writeln!(htif::HostFile::stdout(), "Encoded size: {} bits ({} bytes)", 
             encoded.len(), encoded_bytes).unwrap();
    writeln!(htif::HostFile::stdout(), "Compression ratio: {:.2}x", compression_ratio).unwrap();

    // Decode
    let decode_benchmark = start_benchmark();
    let decoded = coder.decode(&encoded);
    print_benchmark_data(decode_benchmark);

    let decoded_string = String::from_utf8(decoded.clone()).unwrap();

    // Verify
    if TEST_TEXT == decoded.as_slice() {
        writeln!(htif::HostFile::stdout(), "Verification: SUCCESS").unwrap();
        writeln!(htif::HostFile::stdout(), "{}", decoded_string).unwrap();
    } else {
        writeln!(htif::HostFile::stdout(), "Verification: FAILED").unwrap();
        panic!("Huffman decode mismatch!");
    }

    // Print some codes
    let codes = coder.generate_codes();
    writeln!(htif::HostFile::stdout(), "\nSample codes:").unwrap();
    for ch in b"Huffman" {
        if let Some(code) = &codes[*ch as usize] {
            write!(htif::HostFile::stdout(), "  '{}': ", *ch as char).unwrap();
            for &bit in code {
                write!(htif::HostFile::stdout(), "{}", if bit { '1' } else { '0' }).unwrap();
            }
            writeln!(htif::HostFile::stdout(), "").unwrap();
        }
    }

    exit();
}
