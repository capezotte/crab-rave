use teloxide::{prelude::*, utils::command::BotCommands};
use std::fs;
use std::error::Error;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    prep_markov();
    let bot = Bot::from_env().auto_send();
    
    teloxide::commands_repl(bot, answer, Command::ty()).await;
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase")]
enum Command {
    #[command()]
    Cu(String),
}

async fn answer(
    bot: AutoSend<Bot>,
    message: Message,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {

    if let Some(reply) = markov(5, None) {
        bot.send_message(message.chat.id, reply).await?;
    } else {
        bot.send_message(message.chat.id, format!("Can't find '{}' in database.", "s")).reply_to_message_id(message.id).await?;
    }

    Ok(())
}

fn prep_markov() {
    // Check if we've already init
    log::info!("Readying markov chain...");
    if let Ok(connection) = sqlite::open("./wmbr.sqlite") {
        let mut found = false;
        match connection.iterate("SELECT name FROM sqlite_master WHERE name = 'pairs'", |_| { found = true; false }) {
            Ok(()) | Err( sqlite::Error { code: Some(4), .. } ) => {
                if found {
                    log::info!("Using initialized db");
                    return
                }
            }
            Err(x) => { panic!("{:?}", x) }
        }
    }
    // Else init
    log::info!("Initting markov...");
    let connection = sqlite::open(":memory:").unwrap();

    // Initialize
    connection.execute("CREATE TABLE begin (word TEXT NOT NULL);").unwrap(); // words that can begin
    connection
        .execute("CREATE TABLE pairs (word1 TEXT NOT NULL, word2 TEXT);").unwrap(); // pairs. If second word is null, ending.
    let mut begin_stmt = connection.prepare("INSERT INTO begin VALUES (?);").unwrap();
    let mut add_stmt = connection.prepare("INSERT INTO pairs VALUES (?, ?);").unwrap();
    for line in fs::read_to_string("wmbr.db").expect("Can't open db").lines() {
        println!("reading: {line}");
        if line.starts_with(" ") {
            begin_stmt.bind(1, line.trim()).unwrap();
            begin_stmt.next().unwrap();
            begin_stmt.reset().unwrap();
        } else {
            let mut split_str = line.split_whitespace();
            add_stmt.bind(1, split_str.next().unwrap_or("")).unwrap();
            add_stmt.bind(2, split_str.next()).unwrap();
            add_stmt.next().unwrap();
            add_stmt.reset().unwrap();
        }
    }
    log::info!("Create index...");
    connection.execute("CREATE INDEX idx_word1 ON pairs (word1);").unwrap();
    log::info!("Saving markov to disk...");
    connection.execute("VACUUM INTO './wmbr.sqlite';").unwrap();
    log::info!("Done. Starting bot.");
}

fn markov(len: i32, starting: Option<&str>) -> Option<String> {
    use sqlite::State;
    let connection = sqlite::open("./wmbr.sqlite").unwrap();

    let mut cur_word = if let Some(init) = starting {
        init.to_owned()
    } else {
        let mut begin_stmt = connection.prepare("SELECT * FROM begin ORDER BY random();").unwrap();
        if begin_stmt.next().unwrap() == State::Done {
            panic!("Unitialized DB");
        }
        begin_stmt.read::<String>(0).unwrap()
    };

    let mut ret = cur_word.clone();
    // Fill up most words
    {
        let mut next_word_stmt = connection.prepare("SELECT word2 FROM pairs WHERE word1 = ? AND word2 IS NOT NULL ORDER BY random();").unwrap();
        let mut i = len;
        while i > 0 {
            next_word_stmt.bind(1, cur_word.as_str()).unwrap();
            next_word_stmt.next().unwrap();
            if let Some(new_word) = next_word_stmt.read::<Option<String>>(0).unwrap() {
                ret.push_str(" ");
                ret.push_str(&new_word);
                cur_word = new_word.clone();
                i -= 1;
            } else {
                break;
            }
            next_word_stmt.reset().unwrap();
        }
    }
    // Final words
    { 
        let mut final_word_stmt = connection.prepare("SELECT word2 FROM pairs WHERE word1 = ? ORDER BY random();").unwrap();
        final_word_stmt.bind(1, cur_word.as_str()).unwrap();
        final_word_stmt.next().unwrap();
        while let Some(new_word) = final_word_stmt.read::<Option<String>>(0).unwrap() {
            ret.push_str(" ");
            ret.push_str(&new_word);
            cur_word = new_word.clone();
            final_word_stmt.reset().unwrap();
            final_word_stmt.bind(1, cur_word.as_str()).unwrap();
            final_word_stmt.next().unwrap();
        }
    }

    return Some(ret);
}
