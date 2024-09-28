use crate::optimizer::is_simple_loop;
use crate::parser::Command;

pub fn print_profile(commands: &[Command]) {
    struct LoopData {
        idx: usize,
        num_executions: usize,
    }

    fn print_profile_rec(
        commands: &[Command],
        curr_idx: &mut usize,
        simple_loops: &mut Vec<LoopData>,
        non_simple_loops: &mut Vec<LoopData>,
    ) {
        for command in commands {
            match command {
                Command::IncPointer { amount, count } => {
                    let repr = if *amount == 1 {
                        String::from(">")
                    } else {
                        format!(">{}", amount)
                    };
                    println!("{:>6} : {:^6} : {}", curr_idx, repr, count);
                }
                Command::DecPointer { amount, count } => {
                    let repr = if *amount == 1 {
                        String::from("<")
                    } else {
                        format!("<{}", amount)
                    };
                    println!("{:>6} : {:^6} : {}", curr_idx, repr, count);
                }
                Command::IncData {
                    offset,
                    amount,
                    count,
                } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("({})", offset)
                    };
                    let repr = if *amount == 1 {
                        format!("{}+", offset_str)
                    } else {
                        format!("{}+{}", offset_str, amount)
                    };
                    println!("{:>6} : {:^6} : {}", curr_idx, repr, count);
                }
                Command::DecData {
                    offset,
                    amount,
                    count,
                } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("({})", offset)
                    };
                    let repr = if *amount == 1 {
                        format!("{}-", offset_str)
                    } else {
                        format!("{}-{}", offset_str, amount)
                    };
                    println!("{:>6} : {:^6} : {}", curr_idx, repr, count);
                }
                Command::SetData {
                    offset,
                    value,
                    count,
                } => {
                    let offset_str = if *offset == 0 {
                        String::from("")
                    } else {
                        format!("({})", offset)
                    };
                    println!(
                        "{:>6} : {:^6} : {}",
                        curr_idx,
                        format!("{}={}", offset_str, value),
                        count
                    );
                }
                Command::AddOffsetData {
                    dest_offset,
                    src_offset,
                    multiplier,
                    inverted,
                    count,
                } => {
                    let mut dest_string = String::new();
                    let inverted_str = if *inverted { "-" } else { "" };
                    dest_string
                        .push_str(&format!("{}({}*{})", inverted_str, src_offset, multiplier));
                    println!(
                        "{:>6} : {:^6} : {}",
                        curr_idx,
                        format!("({}+={})", dest_offset, dest_string),
                        count
                    );
                }
                Command::SubOffsetData {
                    dest_offset,
                    src_offset,
                    multiplier,
                    inverted,
                    count,
                } => {
                    let mut dest_string = String::new();
                    let inverted_str = if *inverted { "-" } else { "" };
                    dest_string
                        .push_str(&format!("{}({}*{})", inverted_str, src_offset, multiplier));
                    println!(
                        "{:>6} : {:^6} : {}",
                        curr_idx,
                        format!("({}-={})", dest_offset, dest_string),
                        count
                    );
                }
                Command::Output { count } => {
                    println!("{:>6} : {:^6} : {}", curr_idx, ".", count);
                }
                Command::Input { count } => {
                    println!("{:>6} : {:^6} : {}", curr_idx, ",", count);
                }
                Command::Loop {
                    body,
                    id: _,
                    start_count,
                    end_count,
                } => {
                    let (is_simple, _) = is_simple_loop(command);
                    if is_simple {
                        simple_loops.push(LoopData {
                            idx: *curr_idx,
                            num_executions: *end_count,
                        });
                    } else {
                        non_simple_loops.push(LoopData {
                            idx: *curr_idx,
                            num_executions: *end_count,
                        });
                    }

                    println!("{:>6} : {:^6} : {}", curr_idx, "[", start_count);
                    *curr_idx += 1;

                    // Recursively print the commands inside the loop
                    print_profile_rec(body, curr_idx, simple_loops, non_simple_loops);

                    println!("{:>6} : {:^6} : {}", curr_idx, "]", end_count);
                }
            }
            *curr_idx += 1;
        }
    }

    // Driver for recursive method
    let mut init_index = 0;
    let mut simple_loops: Vec<LoopData> = vec![];
    let mut non_simple_loops: Vec<LoopData> = vec![];
    print_profile_rec(
        &commands,
        &mut init_index,
        &mut simple_loops,
        &mut non_simple_loops,
    );

    simple_loops.sort_by(|a, b| b.num_executions.cmp(&a.num_executions));
    non_simple_loops.sort_by(|a, b| b.num_executions.cmp(&a.num_executions));

    for simple_loop in simple_loops {
        println!(
            "[Simple Loop]     : index {:<6} : executions {}",
            simple_loop.idx, simple_loop.num_executions
        );
    }
    for non_simple_loop in non_simple_loops {
        println!(
            "[Non-simple Loop] : index {:<6} : executions {}",
            non_simple_loop.idx, non_simple_loop.num_executions
        );
    }
}
