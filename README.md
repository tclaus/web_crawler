# Web Crawler

Crawls websites recursively. High Performance, with seed DB and store to index. Written in Rust.

## How it works
Web Crawler is a proof-of-concept written in [Rust](https://www.rust-lang.org). It reads a website from commandline or a seed database and starts to crawl all reachable sites, adds unknown external sites to a database and reads these.
All found sites are added to a elastic search index.

The indexed sites can be accessed by web crawlers companion - the [Search UI](https://github.com/tclaus/search_ui).

## How to setup
This is only valid for Mac - for those who likes windows: Good luck.
1. Install [Rust](https://www.rust-lang.org). The site has a good documentation.
2. Install [Postgres as an App](https://postgresapp.com) - the most easy way to get Postgres up and running on a developers machine.
3. Install and run [Postgres Admin](https://www.pgadmin.org)
4. Install [Elastic Search](https://www.elastic.co/de/downloads/elasticsearch) and Kibana (optional)
5. Start all these (Run PostgresApp, Elastic Search)


Now you have the Rust language installed and all databases.
You just need a Postgres Database instance and a seed table. Open Postgres Admin UI and create a Database named "webcrawler_dev".
Copy contents of create_seed_table.sql to a SQL console in the admin and execute.

## How to run
you can now run by
```sh
$ cargo run https://my_seed_site.com
```
or
```sh
$ cargo run
```

In the first case only your site will be crawled. In the second case the seed database is queried, all entries are scanned and will be crawled. If any external sites are linked, these will be added to this database. So it will grow.

If you have the [Search UI](https://github.com/tclaus/search_ui) installed from Github you now can open a search console on http://localhost:3000 and check your results

### TODO / How to help

- Stabilize. After some thousands crawled sites it hangs
- Make some real logs. Don't let the log grow infinitely
- Improve read of seed-table. Update with last crawled date, add a 'max_deeph' factor
- Add an ignore table. Some external links should never be crawled like Add-Sites, Porn, Illegal stuff and so on. Ignore them early.


### License

MIT
---
Free to use. Be polite and reference to its original creator


Written by Thorsten Claus, Dortmund, Germany
