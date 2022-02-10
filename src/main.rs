use std::io::{stdin, BufRead, BufReader};
use clap::{App, Arg, ArgGroup};
use std::fs::File;

// フィールド指定がどの形式で行われているか
enum FieldSpecification {
    ByFieldNumber,       // フィールド番号
    ByFieldName,         // フィールド名
    ByCharCount,         // 文字数
}

fn main() {
    // オプション定義
    let app = App::new("ncut")
                .version("0.1.0")
                .about("cut by field name")
                .author("Mugi_GrainP")
                .arg(Arg::with_name("file")
                     .help("ファイル名")
                 )
                .arg(Arg::with_name("delim")
                     .help("区切り記号 (デフォルトはTAB)")
                     .short("d")
                     .long("delimiter")
                     .takes_value(true)
                 )
                .arg(Arg::with_name("characters")
                     .help("文字数で切り出す")
                     .short("c")
                     .long("characters")
                     .takes_value(true)
                 )
                .arg(Arg::with_name("field")
                     .help("フィールド識別子")
                     .short("f")
                     .long("field")
                     .takes_value(true)
                 )
                .arg(Arg::with_name("field_by_title")
                     .help("フィールド識別子 (タイトル行を利用)")
                     .short("t")
                     .long("field-by-title")
                     .takes_value(true)
                 )
                .group(ArgGroup::with_name("field_specification")
                       .args(&["characters", "field", "field_by_title"])
                       .required(true)
                 );


    // オプション解析
    let matches = app.get_matches();

    // 区切り記号を設定
    // オプションによる指定がない場合のデフォルト値はTAB
    let delimiter = match matches.value_of("delim") {
        Some(o) => if matches.is_present("characters") {
            ""
        } else {
            o
        },
        None => if matches.is_present("characters") {
            ""
        } else {
            "\t" 
        },
    };

    // フィールド名によるフィールド選択の場合、その選択値を取得
    // フィールド番号（文字数・バイト数）指定が優先される
    let (fnum, fname, charcount) = (
        matches.is_present("field"),
        matches.is_present("field_by_title"),
        matches.is_present("characters"),
    );
    let viewfield_str: (FieldSpecification, &str) = match (fnum, fname, charcount) {
        (true, false, false) => (FieldSpecification::ByFieldNumber, matches.value_of("field").unwrap()),
        (false, true, false) => (FieldSpecification::ByFieldName,   matches.value_of("field_by_title").unwrap()),
        (false, false, true) => (FieldSpecification::ByCharCount,   matches.value_of("characters").unwrap()),
        _ => unreachable!(),
    };

    // ファイル名が指定されている場合はそのファイルを開く
    // ファイル名が指定されていない場合は標準入力を開く
    match matches.value_of("file") {
        Some(path) => {
            match File::open(path) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    read_and_output(reader, delimiter, viewfield_str);
                },
                Err(e) => println!("{}: {}", path, e),
            }
        },
        None => {
            let stdin = stdin();
            let reader = stdin.lock();
            read_and_output(reader, delimiter, viewfield_str);
        },
    }
}

// ファイル読み込み表示
fn read_and_output<R: BufRead>(mut reader: R, delimiter: &str, viewfield_str: (FieldSpecification, &str)) {
    match viewfield_str.0 {
        FieldSpecification::ByCharCount => {
            for line in reader.lines() {
                let row = line.unwrap();
                //  文字数を算出
                let char_count = row.split(delimiter).collect::<Vec<_>>().len();
                let viewfield = set_viewfield(char_count, viewfield_str.1);

                split_and_print(&row, delimiter, &viewfield);
            }
        },

        FieldSpecification::ByFieldName | FieldSpecification::ByFieldNumber => {
            // 1行目がフィールド名である可能性に備え、また、フィールド数を算出するため、別途読み出す
            let mut first_line = String::new();
            reader.read_line(&mut first_line).expect("Can't read");
            first_line = first_line.trim_end().to_string();

            // フィールド数を算出
            let field_count = first_line.split(delimiter).collect::<Vec<_>>().len();

            // 出力するフィールドを指定
            // フィールド数が1の時（区切り記号によって区切られなかった時）はフィールド全体を出力
            let viewfield = match viewfield_str.0 {
                FieldSpecification::ByFieldNumber => set_viewfield(field_count, viewfield_str.1),
                FieldSpecification::ByFieldName   => set_viewfield(field_count,
                                                     &make_viewfield_str(first_line.split(delimiter).collect(), viewfield_str.1)),
                FieldSpecification::ByCharCount   => unreachable!(),
            };

            // 区切り記号、フィールド指定をもとにデータを標準出力に出力する
            split_and_print(&first_line, delimiter, &viewfield);
            for line in reader.lines() {
                let row = line.unwrap();
                split_and_print(&row, delimiter, &viewfield);
            }
        },
    }
}

// 区切り記号、フィールド指定をもとにデータを標準出力に出力する部分の本体
fn split_and_print(row: &str, delimiter: &str, viewfield: &Vec<bool>) {
    let mut output: Vec<&str> = row.split(delimiter).collect();
    let mut i = 0;

    // 文字数で切り出す場合に限り、先頭の空要素を除去
    if delimiter == "" {
        output.remove(0);
    }

    // viewfieldがtrueの値のみを残す
    output.retain(|_| (viewfield[i], i += 1).0);
    let output_line = output.join(delimiter);

    println!("{}", output_line);
}

// 出力フィールド指定子を解釈し、各フィールドを出力するかどうか真偽値Vectorで返す
fn set_viewfield(field_count: usize, field_str: &str) -> Vec<bool> {
    match field_count {
        // フィールド数が1の時（区切り記号によって区切られなかった時）はフィールド全体を出力（唯一のフィールドをTrueに）
        1 => vec![true],
        _ => {
            let token: Vec<&str> = field_str.split(',').collect();
            let mut viewfield: Vec<bool> = vec![false; field_count];

            for t in token {
                match t {
                    x if x.find('-').is_some()  => {
                        let from_to: Vec<&str> = x.split('-').collect();

                        let fstart = usize::from_str_radix(from_to[0], 10).unwrap() ;
                        let fend = usize::from_str_radix(from_to[1], 10).unwrap_or(field_count);

                        let actual_start = if fstart <= field_count {
                            fstart
                        } else {
                            field_count
                        };

                        let actual_end = if fend <= field_count {
                            fend
                        } else {
                            field_count
                        };

                        for i in (actual_start - 1)..=(actual_end - 1) {
                            viewfield[i] = true;
                        }
                    },
                    _ => {
                        let n = usize::from_str_radix(t, 10).unwrap();
                        if n <= field_count {
                            viewfield[n - 1] = true;
                        }
                    },
                }
            }

            viewfield
        }
    }
}

fn make_viewfield_str(header_list: Vec<&str>, fields: &str) -> String {
    let field_list: Vec<&str> = fields.split(',').collect();
    let mut viewfield: Vec<String> = vec![];

    for f in field_list {
        let mut header_list_iter = header_list.iter();
        match header_list_iter.position(|&x| x == f) {
            Some(idx) => viewfield.push((idx + 1).to_string()),
            None => (),
        }
    }

    viewfield.join(",")
}

