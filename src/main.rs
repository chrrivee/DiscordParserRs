use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;

#[derive(Parser)]
#[command(name = "deleted-user-analyzer")]
#[command(about = "Efficiently analyze JSON files for deleted user mentions")]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, default_value = "3")]
    min_word_length: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Message {
    message_id: String,
    content: String,
    timestamp: String,
    author_name: String,
    author_nickname: String,
    author_id: String,
    mentioned_user_name: Option<String>,
    mentioned_user_nickname: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
struct AuthorAnalysis {
    author_id: String,
    author_name: String,
    author_nickname: String,
    total_messages_to_deleted_user: usize,
    unique_message_count: usize,
    word_frequency: BTreeMap<String, usize>,
    most_common_words: Vec<(String, usize)>,
}

#[derive(Serialize, Debug)]
struct AnalysisResult {
    total_messages: usize,
    messages_to_deleted_users: usize,
    unique_authors: usize,
    authors_analysis: Vec<AuthorAnalysis>,
    global_word_frequency: BTreeMap<String, usize>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Starting analysis of: {}", args.input);
    }

    let file_stuff = fs::read_to_string(&args.input)?;
    let msgs: Vec<Message> = serde_json::from_str(&file_stuff)?;

    if args.verbose {
        println!("Loaded {} messages", msgs.len());
    }

    let deleted_msgs: Vec<Message> = msgs
        .into_par_iter()
        .filter(|msg| {
            msg.mentioned_user_name
                .as_ref()
                .map_or(false, |name| name.to_lowercase().contains("deleted user"))
                || msg
                    .mentioned_user_nickname
                    .as_ref()
                    .map_or(false, |nickname| {
                        nickname.to_lowercase().contains("deleted user")
                    })
        })
        .collect();

    if args.verbose {
        println!(
            "Found {} messages mentioning deleted users",
            deleted_msgs.len()
        );
    }

    let mut author_msg_map: HashMap<String, Vec<Message>> = HashMap::new();

    for msg in deleted_msgs {
        author_msg_map
            .entry(msg.author_id.clone())
            .or_insert_with(Vec::new)
            .push(msg);
    }

    for (_, msgs) in author_msg_map.iter_mut() {
        msgs.sort_by(|a, b| a.content.cmp(&b.content));
        msgs.dedup_by(|a, b| a.content == b.content);
    }

    if args.verbose {
        println!("Found {} unique authors", author_msg_map.len());
    }

    let analysis_data: Vec<AuthorAnalysis> = author_msg_map
        .into_iter()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|(author_id, msgs)| {
            let total_msgs = msgs.len();
            let author_name = msgs[0].author_name.clone();
            let author_nickname = msgs[0].author_nickname.clone();

            let mut word_freq_map: HashMap<String, usize> = HashMap::with_capacity(64);

            for msg in &msgs {
                let words = tokenize_content(&msg.content, args.min_word_length);
                for word in words {
                    *word_freq_map.entry(word).or_insert(0) += 1;
                }
            }

            let word_frequency: BTreeMap<String, usize> = word_freq_map.into_iter().collect();
            let mut common_words: Vec<(String, usize)> = word_frequency
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();

            common_words.sort_by(|a, b| b.1.cmp(&a.1));
            common_words.truncate(10);

            AuthorAnalysis {
                author_id,
                author_name,
                author_nickname,
                total_messages_to_deleted_user: total_msgs,
                unique_message_count: msgs.len(),
                word_frequency,
                most_common_words: common_words,
            }
        })
        .collect();

    let mut global_freq: HashMap<String, usize> = HashMap::with_capacity(256);
    for analysis in &analysis_data {
        for (word, count) in &analysis.word_frequency {
            *global_freq.entry(word.clone()).or_insert(0) += count;
        }
    }
    let global_word_frequency: BTreeMap<String, usize> = global_freq.into_iter().collect();

    let result = AnalysisResult {
        total_messages: analysis_data
            .iter()
            .map(|a| a.total_messages_to_deleted_user)
            .sum(),
        messages_to_deleted_users: analysis_data.iter().map(|a| a.unique_message_count).sum(),
        unique_authors: analysis_data.len(),
        authors_analysis: analysis_data,
        global_word_frequency,
    };

    display_results(&result, args.verbose);

    if let Some(output_path) = args.output {
        let output_json = serde_json::to_string_pretty(&result)?;
        fs::write(&output_path, output_json)?;
        println!("Results saved to: {}", output_path);
    }

    Ok(())
}

fn tokenize_content(content: &str, min_len: usize) -> Vec<String> {
    content
        .to_lowercase()
        .split_whitespace()
        .filter_map(|w| {
            let clean_w: String = w.chars().filter(|c| c.is_alphanumeric()).collect();

            if clean_w.len() >= min_len {
                Some(clean_w)
            } else {
                None
            }
        })
        .collect()
}

fn display_results(result: &AnalysisResult, verbose: bool) {
    println!("\nANALYSIS RESULTS");
    println!("==================");
    println!("Total messages to deleted users: {}", result.total_messages);
    println!(
        "Unique messages (after deduplication): {}",
        result.messages_to_deleted_users
    );
    println!("Unique authors: {}", result.unique_authors);

    if verbose {
        println!("\nAUTHORS ANALYSIS");
        println!("===================");

        let mut sorted_auth = result.authors_analysis.clone();
        sorted_auth.sort_by(|a, b| {
            b.total_messages_to_deleted_user
                .cmp(&a.total_messages_to_deleted_user)
        });

        for (i, auth) in sorted_auth.iter().enumerate().take(10) {
            println!(
                "\n{}. {} ({})",
                i + 1,
                auth.author_name,
                auth.author_nickname
            );
            println!("   Author ID: {}", auth.author_id);
            println!(
                "   Messages to deleted user: {}",
                auth.total_messages_to_deleted_user
            );
            println!("   Unique messages: {}", auth.unique_message_count);

            if !auth.most_common_words.is_empty() {
                println!("   Most common words:");
                for (word, count) in auth.most_common_words.iter().take(5) {
                    println!("     - {}: {}", word, count);
                }
            }
        }

        println!("\nGLOBAL WORD FREQUENCY (TOP 20)");
        println!("==================================");
        let mut global_w: Vec<(String, usize)> = result
            .global_word_frequency
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        global_w.sort_by(|a, b| b.1.cmp(&a.1));

        for (i, (word, count)) in global_w.iter().enumerate().take(20) {
            println!("{}. {}: {}", i + 1, word, count);
        }
    }

    println!("\nAnalysis complete!");
    println!("Use --output <filename.json> to save detailed results");
    println!("Use --verbose for detailed output");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_content() {
        let content = "Hello, World! This is a test.";
        let tokens = tokenize_content(content, 3);
        assert_eq!(tokens, vec!["hello", "world", "this", "test"]);
    }

    #[test]
    fn test_tokenize_content_with_numbers() {
        let content = "Test123 with numbers 456!";
        let tokens = tokenize_content(content, 3);
        assert_eq!(tokens, vec!["test123", "with", "numbers", "456"]);
    }

    #[test]
    fn test_tokenize_content_filters_short_words() {
        let content = "a an the is to of";
        let tokens = tokenize_content(content, 3);
        assert_eq!(tokens, vec!["the"]);
    }

    #[test]
    fn test_tokenize_content_min_length() {
        let content = "a bb ccc dddd";
        let tokens = tokenize_content(content, 2);
        assert_eq!(tokens, vec!["bb", "ccc", "dddd"]);
    }
}
