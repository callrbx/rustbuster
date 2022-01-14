use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

pub struct Wordlist {
    pub path: PathBuf,
    pub base_count: usize,
    pub total_count: usize,
    reader: BufReader<File>,
    pub prepend: Vec<String>,
    pub append: Vec<String>,
    pub swap: Vec<String>,
    pub extensions: Vec<String>,
    word_perms: VecDeque<String>,
}

fn count_lines<R: io::Read>(handle: R) -> usize {
    let mut reader = BufReader::new(handle);
    let mut count = 0;
    let mut line: Vec<u8> = Vec::new();
    while match reader.read_until(b'\n', &mut line) {
        Ok(n) if n > 0 => true,
        Err(e) => {
            eprintln!("[!] Failed to read from wordlist: {}", e);
            std::process::exit(-1);
        }
        _ => false,
    } {
        if *line.last().unwrap() == b'\n' {
            count += 1;
        };
    }
    count
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

impl Wordlist {
    pub fn new(
        path: &PathBuf,
        prepend: Option<String>,
        append: Option<String>,
        swap: Option<String>,
        extensions: Option<String>,
    ) -> Self {
        let pre_strs = prepend
            .unwrap_or(String::new())
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let pre_len = pre_strs.len();
        let app_strs = append
            .unwrap_or(String::new())
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let app_len = app_strs.len();
        let ext_strs = extensions
            .unwrap_or(String::new())
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let ext_len = ext_strs.len();
        let swap_strs = swap
            .unwrap_or(String::new())
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let swap_len = swap_strs.len();
        let word_count = count_lines(std::fs::File::open(&path).unwrap());
        Self {
            path: path.clone(),
            base_count: word_count,
            reader: BufReader::new(File::open(path).unwrap()),
            prepend: pre_strs,
            append: app_strs,
            swap: swap_strs,
            extensions: ext_strs,
            total_count: word_count * pre_len * app_len * swap_len * ext_len,
            word_perms: VecDeque::new(),
        }
    }
}

impl Iterator for Wordlist {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.word_perms.is_empty() {
            let mut base_word = String::new();
            match self.reader.read_line(&mut base_word) {
                Ok(n) => {
                    if n != 0 {
                        trim_newline(&mut base_word);
                        let mut base_words: Vec<String> = Vec::new();

                        if base_word.contains("{SWAP}") {
                            for s in &self.swap {
                                base_words.push(base_word.clone().replace("{SWAP}", &s))
                            }
                        } else {
                            self.word_perms.push_back(base_word.clone());
                            base_words.push(base_word);
                        }

                        for b in base_words {
                            for p in &self.prepend {
                                self.word_perms.push_back(format!("{}{}", p, b));
                                for a in &self.append {
                                    self.word_perms.push_back(format!("{}{}{}", p, b, a));
                                }
                            }
                        }

                        for w in self.word_perms.clone() {
                            for e in &self.extensions {
                                self.word_perms.push_back(format!("{}{}", w, e));
                            }
                        }
                    } else {
                        return None;
                    }
                }
                Err(_) => return None,
            }
        }
        self.word_perms.pop_front()
    }
}
