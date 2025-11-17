#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use riscv_benchmarks::*;
use riscv_rt::entry;
use core::fmt::Write;

// Adjacency list representation of a graph
struct Graph {
    num_vertices: usize,
    adj_list: Vec<Vec<usize>>,
}

impl Graph {
    fn new(num_vertices: usize) -> Self {
        let mut adj_list = Vec::new();
        for _ in 0..num_vertices {
            adj_list.push(Vec::new());
        }
        Graph { num_vertices, adj_list }
    }

    fn add_edge(&mut self, src: usize, dst: usize) {
        self.adj_list[src].push(dst);
    }

    // Breadth-First Search
    fn bfs(&self, start: usize) -> Vec<usize> {
        let mut visited = vec![false; self.num_vertices];
        let mut order = Vec::new();
        let mut queue = VecDeque::new();

        visited[start] = true;
        queue.push_back(start);

        while let Some(v) = queue.pop_front() {
            order.push(v);

            for &neighbor in &self.adj_list[v] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }

        order
    }

    // Depth-First Search (iterative)
    fn dfs_iterative(&self, start: usize) -> Vec<usize> {
        let mut visited = vec![false; self.num_vertices];
        let mut order = Vec::new();
        let mut stack = Vec::new();

        stack.push(start);

        while let Some(v) = stack.pop() {
            if !visited[v] {
                visited[v] = true;
                order.push(v);

                // Push neighbors in reverse order to maintain left-to-right traversal
                for &neighbor in self.adj_list[v].iter().rev() {
                    if !visited[neighbor] {
                        stack.push(neighbor);
                    }
                }
            }
        }

        order
    }

    // Depth-First Search (recursive)
    fn dfs_recursive(&self, start: usize) -> Vec<usize> {
        let mut visited = vec![false; self.num_vertices];
        let mut order = Vec::new();
        self.dfs_helper(start, &mut visited, &mut order);
        order
    }

    fn dfs_helper(&self, v: usize, visited: &mut [bool], order: &mut Vec<usize>) {
        visited[v] = true;
        order.push(v);

        for &neighbor in &self.adj_list[v] {
            if !visited[neighbor] {
                self.dfs_helper(neighbor, visited, order);
            }
        }
    }

    // Detect cycle using DFS
    fn has_cycle(&self) -> bool {
        let mut visited = vec![false; self.num_vertices];
        let mut rec_stack = vec![false; self.num_vertices];

        for v in 0..self.num_vertices {
            if !visited[v] && self.has_cycle_helper(v, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    fn has_cycle_helper(&self, v: usize, visited: &mut [bool], rec_stack: &mut [bool]) -> bool {
        visited[v] = true;
        rec_stack[v] = true;

        for &neighbor in &self.adj_list[v] {
            if !visited[neighbor] {
                if self.has_cycle_helper(neighbor, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack[neighbor] {
                return true;
            }
        }

        rec_stack[v] = false;
        false
    }

    // Topological sort (Kahn's algorithm)
    fn topological_sort(&self) -> Option<Vec<usize>> {
        let mut in_degree = vec![0; self.num_vertices];

        // Calculate in-degrees
        for v in 0..self.num_vertices {
            for &neighbor in &self.adj_list[v] {
                in_degree[neighbor] += 1;
            }
        }

        let mut queue = VecDeque::new();
        for (v, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(v);
            }
        }

        let mut order = Vec::new();
        while let Some(v) = queue.pop_front() {
            order.push(v);

            for &neighbor in &self.adj_list[v] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        if order.len() == self.num_vertices {
            Some(order)
        } else {
            None // Graph has a cycle
        }
    }
}

// Create a test directed acyclic graph
fn create_test_dag() -> Graph {
    let mut graph = Graph::new(6);

    // Create a DAG with edges
    graph.add_edge(5, 2);
    graph.add_edge(5, 0);
    graph.add_edge(4, 0);
    graph.add_edge(4, 1);
    graph.add_edge(2, 3);
    graph.add_edge(3, 1);

    graph
}

#[entry]
fn main() -> ! {
    init_heap();

    let graph = create_test_dag();

    writeln!(htif::HostFile::stdout(), "=== Graph Traversal Benchmarks ===").unwrap();

    // BFS
    let benchmark_data = start_benchmark();
    let bfs_order = graph.bfs(5);
    print_benchmark_data(benchmark_data);
    write!(htif::HostFile::stdout(), "BFS from vertex 5: ").unwrap();
    for v in &bfs_order {
        write!(htif::HostFile::stdout(), "{} ", v).unwrap();
    }
    writeln!(htif::HostFile::stdout(), "").unwrap();

    // DFS (iterative)
    let benchmark_data = start_benchmark();
    let dfs_iter_order = graph.dfs_iterative(5);
    print_benchmark_data(benchmark_data);
    write!(htif::HostFile::stdout(), "DFS (iterative) from vertex 5: ").unwrap();
    for v in &dfs_iter_order {
        write!(htif::HostFile::stdout(), "{} ", v).unwrap();
    }
    writeln!(htif::HostFile::stdout(), "").unwrap();

    // DFS (recursive)
    let benchmark_data = start_benchmark();
    let dfs_rec_order = graph.dfs_recursive(5);
    print_benchmark_data(benchmark_data);
    write!(htif::HostFile::stdout(), "DFS (recursive) from vertex 5: ").unwrap();
    for v in &dfs_rec_order {
        write!(htif::HostFile::stdout(), "{} ", v).unwrap();
    }
    writeln!(htif::HostFile::stdout(), "").unwrap();

    // Cycle detection
    let benchmark_data = start_benchmark();
    let has_cycle = graph.has_cycle();
    print_benchmark_data(benchmark_data);
    writeln!(htif::HostFile::stdout(), "Has cycle: {}", has_cycle).unwrap();

    // Topological sort
    let benchmark_data = start_benchmark();
    let topo_order = graph.topological_sort();
    print_benchmark_data(benchmark_data);
    match topo_order {
        Some(order) => {
            write!(htif::HostFile::stdout(), "Topological order: ").unwrap();
            for v in &order {
                write!(htif::HostFile::stdout(), "{} ", v).unwrap();
            }
            writeln!(htif::HostFile::stdout(), "").unwrap();
        }
        None => {
            writeln!(htif::HostFile::stdout(), "Cannot perform topological sort (cycle detected)").unwrap();
        }
    }

    exit();
}
