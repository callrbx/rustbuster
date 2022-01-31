# Basic Enumeration Tool in Rust

This tool is intended to be a fully modular, wordlist based enumeration tool.
Please use for CTF purposes only.

Modules currently supported:
- dir - webserver file/directory

## Install
Install is simple; just clone and cargo install!
```
git clone https://github.com/callrbx/rustbuster.git
cd rustbuster
cargo install --path .
```
Note: This requires your $PATH to include the `~/.cargo/bin` folder.

## Modules
### base
The base tool currently has very few options, and most options are handled via the modules.

Sample Help Menu:
```
USAGE:
    rustbuster [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --threads <threads>    thread count [default: 10]
```

### dir
The **dir** module exists to perform webserver enumeration.
In this, it's functionality is fairly similar to dirb, dirbuster, gobuster, etc.
However, due to the use of Rust's Tokio async runtime, it has performance slightly faster than these (in my own limited testing).
Combined with Rust's strength in iterator types, it can produce many specified combinations and additions to the specified wordlist on the fly.


Sample Help Menu:
```
USAGE:
    dir [FLAGS] [OPTIONS] --url <url>

FLAGS:
    -f, --add-slash         add a trailing / to each request
    -e, --expand            display full url
    -h, --help              Prints help information
    -k, --no-tls            ignore invalid tls warnings
    -z, --noprog            disable all progress output
    -n, --disable-status    dont display HTTP status codes
    -q, --quiet             disable normal output
    -l, --show-len          displays the size of response
    -r, --show-redir        displays result of a 301 redirect
    -V, --version           Prints version information
    -v, --verbose           enable verbose output

OPTIONS:
    -a, --append <append>            append wordlist words (csv)
    -x, --extensions <extensions>    extensions to search (csv)
    -p, --prepend <prepend>          prepend wordlist words (csv)
    -s, --swap <swap>                swap in for entries that contain {SWAP} (csv)
        --time <timeout>             request timeout [default: 1]
    -u, --url <url>                  target url
    -w, --wordlist <wordlist>        path to wordlist
```

It currently supports several core features, as well as other more "cosmetic" features.

Major Features:
1. Text Prepend (-p, --prepend)
    - "-p dev,prod,bak" will produce variations on each word in the list -> devword, prodword, bakword
2. Text Append (-a, --append)
- "-a dev,prod,bak" will produce variations on each word in the list -> worddev, wordprod, wordbak
3. Extensions (-x, --extensions)
- In addition to the normal append options, this will add the specified file extensions to the current word
- "-x .txt,.html,.bak" will produce variations on each word in the list -> word.txt, word.html, word.bak
- If this was to be used with the above options, a word for each prepend, append, and extension would be produced
4. Swap Mode (-s, --swap)
- Allows templates in your wordlist - "{SWAP}panel" when used with the -s option will produce words for all specified replacements
- "-s admin,dev" will produce -> adminpanel, devpanel

All of these can be combined with each other, which allows for very adaptable tool usage, and a maximizes reusability of wordlists between targets.

Bellow are some examples of the usage of the **dir** mode; target not listed for obvious reasons.

Basic:
```
❯ rustbuster dir -w test.txt -u $TARGET
----------------------------------------
[*] Mode:       dir
[*] URL:        <redacted>
[*] Wordlist:   test.txt (5 entries)
[*] Count:      5
[*] Threads:    10
[*] Discard:    [404]
[*] Prepend:    [""]
[*] Append:     [""]
[*] Swap:       [""]
[*] Ext:        [""]
----------------------------------------

/education (301)
/education (301)
/education (301)
/education (301)
/education (301)
/education (301)
/accessibility (301)
/accessibility (301)
/accessibility (301)
/accessibility (301)
/accessibility (301)
/accessibility (301)
[00:00:00] ################################       5/5       [-] Enumeration complete
```

Custom Cookies, Headers, and User Agent:
```
❯ rustbuster dir -w test.txt -u $TARGET -H 'test: value' -H 'test2: value2' -A haxor -C cookie1=yes -C cookie2=no` -A "SecretAgent"
```


Quiet + No Progress + No Status:
```
❯ rustbuster dir -w test.txt -u $TARGET -q -z -n
/education
/education
/education
/education
/education
/education
/accessibility
/accessibility
/accessibility
/accessibility
/accessibility
/accessibility
```

Verbose:
```
❯ rustbuster dir -w test.txt -u $TARGET -v
----------------------------------------
[*] Mode:       dir
[*] URL:        <redacted>
[*] Wordlist:   test.txt (5 entries)
[*] Count:      5
[*] Threads:    10
[*] Discard:    [404]
[*] Prepend:    [""]
[*] Append:     [""]
[*] Swap:       [""]
[*] Ext:        [""]
----------------------------------------

Drop: /cgi-bin (404)
Drop: /cgi-bin (404)
Keep: /education (301)
Drop: /cgi-bin (404)
Drop: /cgi-bin (404)
Keep: /education (301)
Drop: /cgi-bin (404)
Keep: /education (301)
Drop: /cgi-bin (404)
Keep: /education (301)
Keep: /education (301)
Keep: /education (301)
Drop: /betsie (404)
Drop: /betsie (404)
Drop: /betsie (404)
Drop: /betsie (404)
Drop: /betsie (404)
Drop: /betsie (404)
Keep: /accessibility (301)
Keep: /accessibility (301)
Keep: /accessibility (301)
Keep: /accessibility (301)
Keep: /accessibility (301)
Keep: /accessibility (301)
Drop: /accesskeys (404)
Drop: /accesskeys (404)
Drop: /accesskeys (404)
Drop: /accesskeys (404)
Drop: /accesskeys (404)
Drop: /accesskeys (404)
Drop: /panel (404)
Drop: /panel (404)
Drop: /panel (404)
Drop: /panel (404)
[00:00:00] ################################       5/5       [-] Enumeration complete
```

Power of Modifiers:
```
❯ cat test.txt
{SWAP}panel
❯ rustbuster dir -w test.txt -u $TARGET -v -s admin,user -p dev,prod -a 1,2,3 -x .txt,.php,.html
----------------------------------------
[*] Mode:       dir
[*] URL:        <redacted>
[*] Wordlist:   test.txt (1 entries)
[*] Count:      36
[*] Threads:    10
[*] Discard:    [404]
[*] Prepend:    ["dev", "prod"]
[*] Append:     ["1", "2", "3"]
[*] Swap:       ["admin", "user"]
[*] Ext:        [".txt", ".php", ".html"]
----------------------------------------

Drop: /prodadminpanel1 (404)
Drop: /devadminpanel2 (404)
Drop: /prodadminpanel (404)
Drop: /devuserpanel (404)
Drop: /devadminpanel3 (404)
Drop: /prodadminpanel3 (404)
Drop: /devuserpanel2 (404)
Drop: /devuserpanel3 (404)
Drop: /produserpanel (404)
Drop: /produserpanel1 (404)
Drop: /produserpanel2 (404)
Drop: /produserpanel3 (404)
Drop: /devadminpanel1 (404)
Drop: /devadminpanel (404)
Drop: /devuserpanel1 (404)
Drop: /prodadminpanel2 (404)
Drop: /devadminpanel.txt (404)
Drop: /devadminpanel.php (404)
Drop: /devadminpanel.html (404)
Drop: /devadminpanel1.php (404)
Drop: /devadminpanel1.txt (404)
Drop: /devadminpanel1.html (404)
Drop: /devadminpanel2.txt (404)
Drop: /devadminpanel2.html (404)
Drop: /devadminpanel2.php (404)
Drop: /devadminpanel3.html (404)
Drop: /prodadminpanel.txt (404)
Drop: /devadminpanel3.txt (404)
Drop: /devadminpanel3.php (404)
Drop: /prodadminpanel.php (404)
Drop: /prodadminpanel.html (404)
Drop: /prodadminpanel1.txt (404)
Drop: /prodadminpanel1.php (404)
Drop: /prodadminpanel1.html (404)
Drop: /prodadminpanel2.txt (404)
Drop: /prodadminpanel2.php (404)
Drop: /prodadminpanel3.html (404)
Drop: /prodadminpanel2.html (404)
Drop: /devuserpanel.php (404)
Drop: /prodadminpanel3.txt (404)
Drop: /devuserpanel.txt (404)
Drop: /prodadminpanel3.php (404)
Drop: /devuserpanel.html (404)
Drop: /devuserpanel1.php (404)
Drop: /devuserpanel1.txt (404)
Drop: /devuserpanel1.html (404)
Drop: /devuserpanel2.txt (404)
Drop: /devuserpanel2.html (404)
Drop: /devuserpanel2.php (404)
Drop: /devuserpanel3.html (404)
Drop: /devuserpanel3.php (404)
Drop: /devuserpanel3.txt (404)
Drop: /produserpanel.txt (404)
Drop: /produserpanel.php (404)
Drop: /produserpanel.html (404)
Drop: /produserpanel1.txt (404)
Drop: /produserpanel1.php (404)
Drop: /produserpanel2.txt (404)
Drop: /produserpanel1.html (404)
Drop: /produserpanel2.php (404)
Drop: /produserpanel2.html (404)
Drop: /produserpanel3.txt (404)
Drop: /produserpanel3.php (404)
Drop: /produserpanel3.html (404)
[00:00:00] ################################      36/36      [-] Enumeration complete
```
As you can see, with even just a minimal wordlist we can produce numerous guesses for enumeration.

Future Features (in no particular order):
- Basic Auth (better method than raw headers)

## TODO Modules
### S3 Bucket
### VHost
### Subdomain
