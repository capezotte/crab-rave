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

For convenience, you can run a line-by-line chat dump through the `mkdb` shell script to generate files like the above automatically.

It'll be converted to an indexed sqlite database stored under `chain.sqlite`.
