#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use riscv_benchmarks::*;
use riscv_rt::entry;
use core::fmt::Write;

// A literal is a variable or its negation
// Positive literal = variable index * 2
// Negative literal = variable index * 2 + 1
type Literal = usize;

// A clause is a disjunction of literals (OR)
type Clause = Vec<Literal>;

// A CNF formula is a conjunction of clauses (AND)
struct CNF {
    num_vars: usize,
    clauses: Vec<Clause>,
}

// Variable assignment: None = unassigned, Some(true/false) = assigned
type Assignment = Vec<Option<bool>>;

impl CNF {
    fn new(num_vars: usize) -> Self {
        CNF {
            num_vars,
            clauses: Vec::new(),
        }
    }

    fn add_clause(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    // Check if a clause is satisfied under current assignment
    fn is_clause_satisfied(&self, clause: &Clause, assignment: &Assignment) -> bool {
        for &lit in clause {
            let var = lit / 2;
            let positive = lit % 2 == 0;
            if let Some(val) = assignment[var] {
                if val == positive {
                    return true;
                }
            }
        }
        false
    }

    // Check if a clause is unit (only one unassigned literal, rest false)
    fn find_unit_clause(&self, assignment: &Assignment) -> Option<Literal> {
        for clause in &self.clauses {
            let mut unassigned_lit = None;
            let mut count = 0;
            let mut satisfied = false;

            for &lit in clause {
                let var = lit / 2;
                let positive = lit % 2 == 0;
                match assignment[var] {
                    Some(val) => {
                        if val == positive {
                            satisfied = true;
                            break;
                        }
                    }
                    None => {
                        unassigned_lit = Some(lit);
                        count += 1;
                    }
                }
            }

            if !satisfied && count == 1 {
                return unassigned_lit;
            }
        }
        None
    }

    // DPLL algorithm with unit propagation
    fn dpll(&self, assignment: &mut Assignment) -> bool {
        // Unit propagation
        while let Some(lit) = self.find_unit_clause(assignment) {
            let var = lit / 2;
            let val = lit % 2 == 0;
            assignment[var] = Some(val);
        }

        // Check if all clauses are satisfied
        let mut all_satisfied = true;
        for clause in &self.clauses {
            if !self.is_clause_satisfied(clause, assignment) {
                all_satisfied = false;
                // Check if clause is unsatisfiable (all literals assigned false)
                let mut any_unassigned = false;
                for &lit in clause {
                    if assignment[lit / 2].is_none() {
                        any_unassigned = true;
                        break;
                    }
                }
                if !any_unassigned {
                    return false; // Conflict
                }
            }
        }

        if all_satisfied {
            return true;
        }

        // Choose unassigned variable
        let var = (0..self.num_vars).find(|&v| assignment[v].is_none());

        match var {
            None => all_satisfied,
            Some(v) => {
                // Try assigning true
                let mut new_assignment = assignment.clone();
                new_assignment[v] = Some(true);
                if self.dpll(&mut new_assignment) {
                    *assignment = new_assignment;
                    return true;
                }

                // Try assigning false
                let mut new_assignment = assignment.clone();
                new_assignment[v] = Some(false);
                if self.dpll(&mut new_assignment) {
                    *assignment = new_assignment;
                    return true;
                }

                false
            }
        }
    }

    fn solve(&self) -> Option<Assignment> {
        let mut assignment = vec![None; self.num_vars];
        if self.dpll(&mut assignment) {
            Some(assignment)
        } else {
            None
        }
    }
}

// Create a test 3-SAT problem
// (x0 OR x1 OR x2) AND (!x0 OR x2 OR x3) AND (x1 OR !x2 OR x3) AND (!x1 OR !x3 OR x0)
fn create_test_formula() -> CNF {
    let mut cnf = CNF::new(4); // 4 variables: x0, x1, x2, x3

    // (x0 OR x1 OR x2)
    cnf.add_clause(vec![0, 2, 4]); // x0=0*2, x1=1*2, x2=2*2

    // (!x0 OR x2 OR x3)
    cnf.add_clause(vec![1, 4, 6]); // !x0=0*2+1, x2=2*2, x3=3*2

    // (x1 OR !x2 OR x3)
    cnf.add_clause(vec![2, 5, 6]); // x1=1*2, !x2=2*2+1, x3=3*2

    // (!x1 OR !x3 OR x0)
    cnf.add_clause(vec![3, 7, 0]); // !x1=1*2+1, !x3=3*2+1, x0=0*2

    cnf
}

#[entry]
fn main() -> ! {
    init_heap();

    let formula = create_test_formula();
    let benchmark_data = start_benchmark();

    let solution = formula.solve();

    print_benchmark_data(benchmark_data);

    match solution {
        Some(assignment) => {
            writeln!(htif::HostFile::stdout(), "SAT: Solution found").unwrap();
            for (i, val) in assignment.iter().enumerate() {
                if let Some(v) = val {
                    writeln!(htif::HostFile::stdout(), "  x{} = {}", i, v).unwrap();
                }
            }
        }
        None => {
            writeln!(htif::HostFile::stdout(), "UNSAT: No solution exists").unwrap();
        }
    }

    exit();
}
