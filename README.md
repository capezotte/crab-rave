# crab rave

Send him `/cu` and he'll spill out nonsense.

Require Sqlite3 and Rust >= 1.61.

# DIY

Set the `TELOXIDE_TOKEN` environment variable to your Telegram bot token.

Upon the first run, the bot will look for a file called `in.txt`, using the following format:

- Lines starting with an empty space, containing a single word, are words that can begin a sentence.
- Lines not starting with an empty space, contaning a single word, are words that can end a sentence.
- Lines not starting with an empty space, containing two words, are a word pair.

For example, the sentence `blazing fast memory safe nonsense generator` will generate:

```plain
 blazing
blazing fast
fast memory
memory safe
safe nonsense
nonsense generator
generator
```

The `contrib` subfolder has the `telegram2crabrave` jq script that can convert a Telegram JSON chat dump into this format, as well as a `mkdb` bash script that works with line-by-line entries.

It'll be converted to an indexed sqlite database stored under `chain.sqlite`.
