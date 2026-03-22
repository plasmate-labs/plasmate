# Plasmate SOM Benchmark Report

Date: 2026-04-05
Engine: plasmate v0.1.0
URLs tested: 100
Successful: 98 (98%)

## Summary

| Metric | Mean | Median | P95 |
|---|---|---|---|
| HTML bytes | 380,099 | 207,211 | 1,345,422 |
| SOM bytes | 23,537 | 19,045 | 68,563 |
| Byte ratio | 30.0x | 10.2x | 98.4x |
| HTML tokens (est) | 95,024 | 51,802 | 336,355 |
| SOM tokens (est) | 5,884 | 4,761 | 17,140 |
| Token ratio | 30.0x | 10.2x | 98.4x |
| Elements found | 136 | 117 | 365 |
| Interactive found | 105 | 89 | 277 |
| Fetch time (ms) | 232 | 162 | 693 |
| Parse+SOM time (ms) | 19 | 16 | 55 |

## Per-URL Results

| URL | HTML bytes | SOM bytes | Ratio | Grade | Elements | Interactive | Fetch ms | Parse ms | Status |
|---|---|---|---|---|---|---|---|---|---|
| www.amazon.com/ | 2,007 | 266 | 7.5x | C | 0 | 0 | 89 | 0 | ok |
| www.ebay.com | 0 | 0 | N/A | F | 0 | 0 | 842 | 0 | error |
| www.etsy.com | 0 | 0 | N/A | F | 0 | 0 | 87 | 0 | error |
| www.walmart.com/ | 326,423 | 15,076 | 21.7x | A | 66 | 53 | 175 | 16 | ok |
| www.target.com/ | 413,672 | 19,203 | 21.5x | A | 111 | 80 | 503 | 22 | ok |
| www.bestbuy.com/ | 558,057 | 20,055 | 27.8x | A | 124 | 89 | 693 | 19 | ok |
| www.newegg.com/ | 680,543 | 36,200 | 18.8x | A | 182 | 141 | 105 | 27 | ok |
| www.zappos.com/ | 1,345,422 | 45,779 | 29.4x | A | 124 | 90 | 701 | 29 | ok |
| www.asos.com/ | 277,258 | 27,455 | 10.1x | B | 161 | 130 | 221 | 30 | ok |
| store.steampowered.com/ | 1,496,832 | 19,272 | 77.7x | A | 99 | 73 | 364 | 43 | ok |
| www.bbc.com/news | 339,241 | 22,021 | 15.4x | A | 113 | 103 | 132 | 25 | ok |
| lite.cnn.com/ | 328,470 | 25,965 | 12.7x | B | 111 | 106 | 115 | 12 | ok |
| www.reuters.com/ | 893,602 | 70,479 | 12.7x | B | 360 | 289 | 170 | 32 | ok |
| www.nytimes.com/ | 1,329,458 | 13,509 | 98.4x | A | 10 | 6 | 220 | 35 | ok |
| www.theguardian.com/international | 1,296,061 | 63,704 | 20.3x | A | 314 | 256 | 242 | 64 | ok |
| text.npr.org/ | 5,995 | 6,307 | 1.0x | F | 36 | 28 | 139 | 3 | ok |
| www.washingtonpost.com/ | 2,764,916 | 79,129 | 34.9x | A | 450 | 282 | 225 | 45 | ok |
| news.ycombinator.com/ | 34,367 | 26,008 | 1.3x | D | 210 | 197 | 320 | 4 | ok |
| arstechnica.com/ | 401,423 | 34,945 | 11.5x | B | 161 | 96 | 206 | 27 | ok |
| www.theverge.com/ | 939,593 | 60,075 | 15.6x | A | 289 | 156 | 102 | 29 | ok |
| old.reddit.com/ | 232,874 | 51,779 | 4.5x | C | 367 | 297 | 591 | 32 | ok |
| x.com/ | 245,535 | 1,501 | 163.6x | A | 0 | 0 | 158 | 4 | ok |
| www.linkedin.com/ | 141,388 | 44,497 | 3.2x | C | 193 | 167 | 225 | 19 | ok |
| news.ycombinator.com/newest | 41,195 | 30,657 | 1.3x | D | 214 | 201 | 73 | 11 | ok |
| lobste.rs/ | 59,206 | 18,358 | 3.2x | C | 117 | 115 | 118 | 15 | ok |
| github.com/rust-lang/rust | 368,039 | 36,574 | 10.1x | B | 235 | 189 | 673 | 35 | ok |
| developer.mozilla.org/en-US/docs/Web/HTML | 177,151 | 53,439 | 3.3x | C | 349 | 277 | 89 | 23 | ok |
| stackoverflow.com/questions | 243,822 | 35,917 | 6.8x | C | 229 | 150 | 167 | 87 | ok |
| pypi.org/ | 22,515 | 13,221 | 1.7x | D | 89 | 62 | 104 | 8 | ok |
| crates.io/ | 3,457 | 998 | 3.5x | C | 0 | 0 | 148 | 0 | ok |
| www.npmjs.com/ | 29,455 | 6,069 | 4.9x | C | 37 | 24 | 97 | 4 | ok |
| docs.rs/ | 17,143 | 7,808 | 2.2x | D | 43 | 38 | 187 | 3 | ok |
| pkg.go.dev/ | 33,470 | 17,491 | 1.9x | D | 128 | 95 | 182 | 7 | ok |
| developer.chrome.com/docs | 157,315 | 16,906 | 9.3x | B | 79 | 77 | 311 | 20 | ok |
| about.readthedocs.com/?ref=app.readthedocs.org | 36,503 | 12,196 | 3.0x | D | 85 | 43 | 335 | 10 | ok |
| www.google.com/ | 187,926 | 1,642 | 114.4x | A | 11 | 8 | 169 | 13 | ok |
| www.bing.com/ | 155,754 | 1,586 | 98.2x | A | 3 | 2 | 226 | 9 | ok |
| duckduckgo.com/ | 390,614 | 11,875 | 32.9x | A | 67 | 54 | 116 | 15 | ok |
| search.brave.com/ | 82,772 | 8,994 | 9.2x | B | 17 | 16 | 159 | 6 | ok |
| search.yahoo.com/ | 149,401 | 9,736 | 15.3x | A | 38 | 31 | 199 | 8 | ok |
| linear.app/ | 2,249,035 | 21,390 | 105.1x | A | 176 | 156 | 202 | 39 | ok |
| vercel.com/ | 944,310 | 22,336 | 42.3x | A | 139 | 98 | 150 | 42 | ok |
| stripe.com/ | 574,993 | 41,013 | 14.0x | B | 193 | 171 | 166 | 28 | ok |
| www.notion.com/ | 285,981 | 19,510 | 14.7x | B | 126 | 88 | 649 | 18 | ok |
| www.figma.com/ | 1,476,534 | 23,227 | 63.6x | A | 127 | 106 | 137 | 29 | ok |
| www.cloudflare.com/ | 982,690 | 47,776 | 20.6x | A | 210 | 172 | 259 | 55 | ok |
| www.digitalocean.com/ | 234,828 | 23,000 | 10.2x | B | 145 | 87 | 76 | 13 | ok |
| render.com/ | 312,537 | 10,770 | 29.0x | A | 63 | 46 | 267 | 17 | ok |
| railway.com/ | 210,972 | 10,721 | 19.7x | A | 65 | 44 | 202 | 16 | ok |
| fly.io/ | 200,794 | 16,383 | 12.3x | B | 115 | 91 | 120 | 13 | ok |
| www.usa.gov/ | 47,231 | 18,438 | 2.6x | D | 105 | 84 | 140 | 7 | ok |
| www.whitehouse.gov/ | 262,411 | 31,467 | 8.3x | B | 174 | 147 | 84 | 19 | ok |
| data.gov/ | 121,568 | 12,143 | 10.0x | B | 71 | 51 | 99 | 11 | ok |
| www.nih.gov/ | 59,592 | 13,903 | 4.3x | C | 73 | 59 | 114 | 11 | ok |
| www.cdc.gov/ | 60,014 | 35,783 | 1.7x | D | 238 | 196 | 109 | 12 | ok |
| accounts.google.com/v3/signin/identifier?continue= | 1,206,598 | 1,395 | 864.9x | A | 5 | 3 | 336 | 35 | ok |
| login.microsoftonline.com/common/oauth2/v2.0/autho | 23,692 | 1,269 | 18.7x | A | 0 | 0 | 249 | 1 | ok |
| auth0.com/ | 374,011 | 28,245 | 13.2x | B | 141 | 122 | 151 | 25 | ok |
| github.com/login | 46,422 | 3,829 | 12.1x | B | 16 | 12 | 94 | 8 | ok |
| account.apple.com/ | 207,211 | 22,831 | 9.1x | B | 1 | 0 | 703 | 9 | ok |
| en.wikipedia.org/wiki/Rust_(programming_language) | 592,036 | 56,599 | 10.5x | B | 325 | 213 | 137 | 64 | ok |
| en.wikipedia.org/wiki/Artificial_intelligence | 1,213,638 | 68,563 | 17.7x | A | 381 | 243 | 49 | 62 | ok |
| en.wikipedia.org/wiki/United_States | 1,708,149 | 69,539 | 24.6x | A | 365 | 245 | 36 | 62 | ok |
| boston.craigslist.org/ | 61,063 | 19,584 | 3.1x | C | 127 | 88 | 825 | 17 | ok |
| www.weather.gov/ | 123,467 | 19,045 | 6.5x | C | 127 | 105 | 116 | 13 | ok |
| www.zillow.com/ | 417,252 | 14,937 | 27.9x | A | 100 | 85 | 403 | 21 | ok |
| www.booking.com/ | 3,962 | 267 | 14.8x | B | 0 | 0 | 261 | 0 | ok |
| www.imdb.com/ | 2,003 | 264 | 7.6x | C | 0 | 0 | 94 | 0 | ok |
| www.rottentomatoes.com/ | 595,584 | 35,501 | 16.8x | A | 231 | 188 | 162 | 48 | ok |
| www.allrecipes.com/ | 439,040 | 58,450 | 7.5x | C | 341 | 317 | 244 | 31 | ok |
| example.com/ | 528 | 738 | 0.7x | F | 4 | 1 | 56 | 0 | ok |
| httpbin.org/ | 9,593 | 1,445 | 6.6x | C | 8 | 5 | 106 | 1 | ok |
| motherfuckingwebsite.com/ | 4,903 | 3,792 | 1.3x | D | 21 | 0 | 167 | 0 | ok |
| info.cern.ch/ | 646 | 1,554 | 0.4x | F | 8 | 4 | 569 | 0 | ok |
| lite.duckduckgo.com/lite/ | 1,863 | 1,394 | 1.3x | D | 4 | 2 | 151 | 0 | ok |
| doc.rust-lang.org/book/ | 22,496 | 4,648 | 4.8x | C | 34 | 15 | 105 | 3 | ok |
| docs.python.org/3/ | 17,802 | 18,947 | 0.9x | F | 138 | 97 | 56 | 6 | ok |
| react.dev/ | 272,471 | 19,540 | 13.9x | B | 114 | 105 | 215 | 23 | ok |
| nextjs.org/docs | 774,488 | 22,614 | 34.2x | A | 143 | 123 | 165 | 28 | ok |
| tailwindcss.com/docs/installation/using-vite | 359,777 | 23,431 | 15.4x | A | 158 | 94 | 115 | 20 | ok |
| kubernetes.io/docs/home/ | 486,113 | 92,139 | 5.3x | C | 1058 | 1018 | 111 | 39 | ok |
| docs.docker.com/ | 90,268 | 9,810 | 9.2x | B | 53 | 43 | 139 | 6 | ok |
| www.postgresql.org/docs/ | 8,540 | 8,335 | 1.0x | D | 65 | 47 | 777 | 3 | ok |
| git-scm.com/docs | 15,994 | 14,133 | 1.1x | D | 117 | 82 | 63 | 6 | ok |
| nginx.org/en/docs/ | 13,824 | 18,128 | 0.8x | F | 134 | 98 | 160 | 4 | ok |
| medium.com/ | 38,188 | 3,426 | 11.1x | B | 4 | 2 | 79 | 4 | ok |
| dev.to/ | 251,004 | 50,326 | 5.0x | C | 329 | 269 | 93 | 29 | ok |
| substack.com/ | 80,398 | 3,714 | 21.6x | A | 19 | 11 | 210 | 4 | ok |
| techcrunch.com/ | 413,562 | 75,198 | 5.5x | C | 430 | 221 | 90 | 30 | ok |
| www.economist.com/ | 754,478 | 57,416 | 13.1x | B | 220 | 162 | 231 | 28 | ok |
| www.khanacademy.org/ | 163,332 | 474 | 344.6x | A | 0 | 0 | 220 | 4 | ok |
| ocw.mit.edu/ | 673,317 | 21,689 | 31.0x | A | 157 | 114 | 76 | 43 | ok |
| www.harvard.edu/ | 164,393 | 17,972 | 9.1x | B | 76 | 55 | 90 | 20 | ok |
| www.stanford.edu/ | 156,428 | 23,880 | 6.6x | C | 144 | 112 | 192 | 15 | ok |
| www.coursera.org/ | 728,359 | 39,410 | 18.5x | A | 198 | 155 | 367 | 42 | ok |
| httpbin.org/html | 3,741 | 571 | 6.6x | C | 2 | 0 | 19 | 0 | ok |
| httpbin.org/headers | 804 | 485 | 1.7x | D | 1 | 0 | 23 | 0 | ok |
| www.iana.org/domains/reserved | 10,817 | 11,456 | 0.9x | F | 81 | 60 | 88 | 4 | ok |
| www.githubstatus.com/ | 407,356 | 16,040 | 25.4x | A | 118 | 89 | 267 | 42 | ok |
| www.whatismybrowser.com/ | 77,739 | 29,065 | 2.7x | D | 163 | 100 | 2088 | 5 | ok |

## Grade Distribution

| Grade | Count | Criteria |
|---|---|---|
| A | 35 | >15x ratio |
| B | 23 | 8-15x ratio |
| C | 20 | 3-8x ratio |
| D | 14 | 1-3x ratio |
| F | 6 | <1x ratio |
