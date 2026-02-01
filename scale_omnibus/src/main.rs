use clap::{Parser, Subcommand};
use scale_omnibus::{
    filter_scales, find_scales_by_origin, find_scales_with_intervals_greater_than,
    find_scales_with_up_down_intervals, get_scale, get_scale_names, Scale,
};

#[derive(Parser)]
#[command(name = "scale_omnibus")]
#[command(about = "A CLI for exploring musical scales from The Scale Omnibus")]
#[command(version)]
struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get a scale by name
    Get {
        /// Scale name (case-insensitive)
        name: String,
    },
    /// List all scale names
    List,
    /// Search scales by name pattern
    Search {
        /// Pattern to search for in scale names (case-insensitive)
        pattern: String,
    },
    /// Find scales by origin/culture
    Origin {
        /// Origin to filter by (e.g., "Egypt", "India")
        origin: String,
    },
    /// Find scales with more than N intervals
    Intervals {
        /// Minimum number of intervals (finds scales with MORE than this)
        min: usize,
    },
    /// Find scales with different ascending/descending intervals
    Asymmetric,
    /// Count total number of scales
    Count,
    /// List all unique origins
    Origins,
}

fn print_scale(scale: &Scale) {
    println!("Name: {}", scale.name);

    if let Some(ref intervals) = scale.intervals {
        println!("Intervals: {:?}", intervals);
    }
    if let Some(ref intervals) = scale.intervals_ascending {
        println!("Intervals (ascending): {:?}", intervals);
    }
    if let Some(ref intervals) = scale.intervals_descending {
        println!("Intervals (descending): {:?}", intervals);
    }

    if let Some(ref notes) = scale.notes {
        println!("Notes: {:?}", notes);
    }
    if let Some(ref notes) = scale.notes_ascending {
        println!("Notes (ascending): {:?}", notes);
    }
    if let Some(ref notes) = scale.notes_descending {
        println!("Notes (descending): {:?}", notes);
    }

    if let Some(ref origin) = scale.origin {
        println!("Origin: {}", origin);
    }
}

fn sort_scales(scales: &[Scale]) -> Vec<&Scale> {
    let mut sorted: Vec<_> = scales.iter().collect();
    sorted.sort_by_key(|a| a.name.to_lowercase());
    sorted
}

fn print_scale_list(scales: &[Scale]) {
    let sorted = sort_scales(scales);
    for scale in &sorted {
        print_scale(scale);
        println!();
    }
    eprintln!("Total: {} scales", sorted.len());
}

fn print_scale_list_json(scales: &[Scale]) {
    let sorted = sort_scales(scales);
    let json = serde_json::to_string_pretty(&sorted).expect("Failed to serialize to JSON");
    println!("{}", json);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Get { name } => match get_scale(&name) {
            Ok(scale) => {
                if cli.json {
                    let json =
                        serde_json::to_string_pretty(scale).expect("Failed to serialize to JSON");
                    println!("{}", json);
                } else {
                    print_scale(scale);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },

        Commands::List => {
            let mut names = get_scale_names();
            names.sort();
            if cli.json {
                let json =
                    serde_json::to_string_pretty(&names).expect("Failed to serialize to JSON");
                println!("{}", json);
            } else {
                for name in &names {
                    println!("{}", name);
                }
                eprintln!("\nTotal: {} scales", names.len());
            }
        }

        Commands::Search { pattern } => {
            let pattern_lower = pattern.to_lowercase();
            match filter_scales(|scale| scale.name.to_lowercase().contains(&pattern_lower)) {
                Ok(scales) => {
                    if scales.is_empty() && !cli.json {
                        println!("No scales found matching '{}'", pattern);
                    } else if cli.json {
                        print_scale_list_json(&scales);
                    } else {
                        print_scale_list(&scales);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Origin { origin } => match find_scales_by_origin(&origin) {
            Ok(scales) => {
                if scales.is_empty() && !cli.json {
                    println!("No scales found with origin '{}'", origin);
                } else if cli.json {
                    print_scale_list_json(&scales);
                } else {
                    print_scale_list(&scales);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Intervals { min } => match find_scales_with_intervals_greater_than(min) {
            Ok(scales) => {
                if scales.is_empty() && !cli.json {
                    println!("No scales found with more than {} intervals", min);
                } else if cli.json {
                    print_scale_list_json(&scales);
                } else {
                    print_scale_list(&scales);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Asymmetric => match find_scales_with_up_down_intervals() {
            Ok(scales) => {
                if scales.is_empty() && !cli.json {
                    println!("No asymmetric scales found");
                } else if cli.json {
                    print_scale_list_json(&scales);
                } else {
                    print_scale_list(&scales);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Count => {
            let count = get_scale_names().len();
            println!("{}", count);
        }

        Commands::Origins => match filter_scales(|scale| scale.origin.is_some()) {
            Ok(scales) => {
                let mut origins: Vec<_> = scales
                    .iter()
                    .filter_map(|s| s.origin.as_ref())
                    .collect();
                origins.sort();
                origins.dedup();
                if cli.json {
                    let json =
                        serde_json::to_string_pretty(&origins).expect("Failed to serialize to JSON");
                    println!("{}", json);
                } else {
                    for origin in &origins {
                        println!("{}", origin);
                    }
                    eprintln!("\nTotal: {} unique origins", origins.len());
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}
