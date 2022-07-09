use teloxide::{prelude::*, utils::command::BotCommands};
use std::fs;
use std::error::Error;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    prep_markov();
    log::info!("Done. Starting bot.");
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
    let Command::Cu(s) = command;
    let min_len: u8 = s.parse().unwrap_or(5);

    if let Some(reply) = markov(min_len, None) {
        bot.send_message(message.chat.id, reply).await?;
    } else {
        bot.send_message(message.chat.id, "unable to generate message!1! panic!").reply_to_message_id(message.id).await?;
    }

    Ok(())
}

fn prep_markov() {
    // Check if we've already init
    log::trace!("Readying markov chain...");
    if let Ok(connection) = sqlite::open("./markov.sqlite") {
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
    log::trace!("Initting markov...");
    let connection = sqlite::open(":memory:").unwrap();

    // Initialize
    connection.execute("CREATE TABLE begin (word TEXT NOT NULL);").unwrap(); // words that can begin
    connection
        .execute("CREATE TABLE pairs (word1 TEXT NOT NULL, word2 TEXT);").unwrap(); // pairs. If second word is null, ending.
    let mut begin_stmt = connection.prepare("INSERT INTO begin VALUES (?);").unwrap();
    let mut add_stmt = connection.prepare("INSERT INTO pairs VALUES (?, ?);").unwrap();
    for line in fs::read_to_string("in.txt").expect("Can't open input").lines() {
        if line.starts_with(" ") {
            let new_word = line.trim();
            begin_stmt.bind(1, new_word).unwrap();
            begin_stmt.next().unwrap();
            begin_stmt.reset().unwrap();
            log::trace!("Added word: {new_word}");
        } else {
            let mut split_str = line.split_whitespace();
            let word1 = split_str.next().unwrap_or("");
            let word2 = split_str.next();
            if let Some(w2) = word2 {
                log::trace!("Word pair: {word1} and {w2}");
            } else {
                log::trace!("End word: {word1}");
            }
            add_stmt.bind(1, split_str.next().unwrap_or("")).unwrap();
            add_stmt.bind(2, split_str.next()).unwrap();
            add_stmt.next().unwrap();
            add_stmt.reset().unwrap();
        }
    }
    log::trace!("Create index...");
    connection.execute("CREATE INDEX idx_word1 ON pairs (word1);").unwrap();
    log::trace!("Saving markov to disk...");
    connection.execute("VACUUM INTO './markov.sqlite';").unwrap();
}

fn markov(len: u8, starting: Option<&str>) -> Option<String> {
    use sqlite::State;
    let connection = sqlite::open("./markov.sqlite").unwrap();

    let mut cur_word = if let Some(init) = starting {
        init.to_owned()
    } else {
        let mut begin_stmt = connection.prepare("SELECT * FROM begin ORDER BY random();").unwrap();
        if begin_stmt.next().unwrap() == State::Done {
            panic!("Bad DB: can't select from begin table");
        }
        begin_stmt.read::<String>(0).unwrap()
    };

    let mut ret = cur_word.clone();
    // Fill up most words
    {
        let mut next_word_stmt = connection.prepare("SELECT word2 FROM pairs WHERE word1 = ? AND word2 IS NOT NULL ORDER BY random();").unwrap();
        for i in 0..len {
            log::trace!("Iteration {i} without null");
            next_word_stmt.bind(1, cur_word.as_str()).unwrap();
            next_word_stmt.next().unwrap();
            if let Some(new_word) = next_word_stmt.read::<Option<String>>(0).unwrap() {
                ret.push_str(" ");
                ret.push_str(&new_word);
                cur_word = new_word.clone();
            } else {
                break;
            }
            next_word_stmt.reset().unwrap();
        }
    }
    // Final words
    { 
        let mut final_word_stmt = connection.prepare("SELECT word2 FROM pairs WHERE word1 = ? ORDER BY random();").unwrap();
        loop {
            log::trace!("Extra iteration null");
            final_word_stmt.bind(1, cur_word.as_str()).unwrap();
            final_word_stmt.next().unwrap();
            match final_word_stmt.read::<Option<String>>(0).unwrap() {
                Some(new_word) => {
                    ret.push_str(" ");
                    ret.push_str(&new_word);
                    cur_word = new_word.clone();
                },
                None => {
                    break;
                }
            }
            final_word_stmt.reset().unwrap();
        }
    }

    return Some(ret);
}
