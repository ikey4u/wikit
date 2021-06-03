use crate::elog;
use crate::error::{Context, AnyResult, WikitError};

use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{Read, BufRead, BufReader, BufWriter, Write, Lines};
use std::fs::File;
use std::path::{Path, PathBuf};

use sqlx::postgres::PgPoolOptions;
use nom::number::streaming::{be_u32, le_u32};
use nom::{regex, do_parse, tuple, map_res, take, count, take_until, pair, cond};
use compress::zlib;
use adler::Adler32;
use ripemd128::{Ripemd128, Digest};
use encoding_rs::GB18030;
use chrono::{DateTime, Local};

type NomResult<'a, O> = AnyResult<(&'a [u8], O), nom::Err<WikitError>>;

#[derive(PartialEq)]
pub enum ParseOption {
    OnlyHeader,
}

struct MDXInfo {
    version: u32,
    encoding: String,
    integersz: u32,
    encid: u32,
}

struct MDXSource {
    iter: Lines<BufReader<File>>,
}

impl Iterator for MDXSource {
    type Item = (String, String);
    fn next(&mut self) -> Option<Self::Item> {
        let (mut word, mut meaning) = (String::new(), String::new());
        loop {
            match self.iter.next() {
                Some(line) => match line {
                    Ok(line) => {
                        if line.trim() == "</>" {
                            break;
                        }
                        if word.len() == 0 {
                            word = line.trim().to_string();
                        } else {
                            meaning.push_str(line.as_str().trim());
                        }
                    },
                    Err(e) => return None
                },
                None => return None
            }
        }
        return Some((word, meaning));
    }
}

fn bytes_to_u64(buf: &[u8], be: bool) -> u64 {
    let start = 0;
    let end = if buf.len() > 8 { 8 } else { buf.len() };
    let mut value = 0u64;
    for i in start..end {
        let byte = if be {
            buf[i]
        } else {
            buf[end - 1 - i]
        };
        value = (value << 8) | {byte as u64};
    }
    value
}

fn mdx_decrypt(mut cipher: Vec<u8>, key: Vec<u8>) -> AnyResult<Vec<u8>> {
    let mut previous = 0x36;
    for i in 0..cipher.len() {
        let (mut cipher_char, key_char) = (cipher[i] as usize, key[i % key.len()] as usize);
        cipher_char = (cipher_char >> 4 | cipher_char << 4) & 0xFF;
        cipher_char = cipher_char ^ previous ^ (i & 0xFF) ^ key_char;
        previous = cipher[i] as usize;
        cipher[i] = cipher_char as u8;
    }
    Ok(cipher)
}

fn mdx_decode(mdxinfo: &MDXInfo, buffer: &[u8]) -> AnyResult<String> {
    let word_text = match mdxinfo.encoding.as_str() {
        "GB18030" => {
            let (word_text, _encoding_used, _has_malformed_chars) = GB18030.decode(buffer);
            word_text.to_string()
        }
        _ => {
            String::from_utf8(buffer.to_vec())
            .context(elog!("invalid utf8 word text"))?
            .replace("\x00", "")
        }
    };
    Ok(word_text)
}

impl MDXInfo {
    fn new(meta: HashMap<String, String>) -> AnyResult<Self> {
        let version: f64 = if let Some(version) = meta.get("GeneratedByEngineVersion") {
            version.parse().context(elog!("Cannot parse version: {}", version))?
        } else {
            return Err(elog!("[!] No engine version"));
        };
        let version = (version * 10.0) as u32;

        let integersz = if version < 20 {
            4
        } else {
            8
        };

        let encoding = if let Some(encoding) = meta.get("Encoding") {
            if encoding.contains("GBK") || encoding.contains("GB2312") {
                String::from("GB18030")
            } else {
                encoding.to_owned()
            }
        } else {
            String::from("UTF-8")
        };

        let encid = if let Some(encid) = meta.get("Encrypted") {
            match encid.as_str() {
                "Yes" | "yes" => 1,
                "No" | "no" => 0,
                _ => {
                    if let Ok(encid) = encid.parse::<u32>() {
                        encid
                    } else {
                        0
                    }
                }
            }
        } else {
            0
        };

        Ok(MDXInfo {
            version,
            encoding,
            integersz,
            encid,
        })
    }
}

struct MdxPacket<'a> {
    packtype: u32,
    adler32: u32,
    data: &'a [u8],
    remain: &'a [u8],
}

impl<'a> MdxPacket<'a> {
    fn new(buf: &'a[ u8], packetsz: u64) -> AnyResult<Self> {
        let r: NomResult<_> = tuple!(buf, le_u32, take!(4), take!(packetsz - 8));
        let (remain, (packtype, adler32buf, data)) = r.context(elog!("parse mdx packet header"))?;
        let adler32 = u32::from_be_bytes(
            adler32buf.try_into().context(elog!("failed to get adler32"))?
        );
        Ok(MdxPacket {
            packtype,
            adler32,
            data,
            remain,
        })
    }
}

pub fn parse_mdx(mdxpath: &str, option: Option<ParseOption>) -> AnyResult<Vec<(String, String)>> {
    let mut buf = Vec::new();
    File::open(mdxpath).context(elog!("failed to open {}", mdxpath))?
        .read_to_end(&mut buf)
        .context(elog!("cannot read mdx file {}", mdxpath))?;

    println!("[+] Parse header ...");
    let mdict_header: NomResult<_> = do_parse!(&buf[..],
        size: be_u32 >>
        meta: map_res!(take!(size + 4),
            |x: &[u8]| -> AnyResult<HashMap<String, String>> {
                let (metabuf, adler32buf) = x.split_at(x.len() - 4);

                let mut adler = Adler32::new();
                adler.write_slice(metabuf);
                let meta_adler32_want = adler.checksum();
                let meta_adler32_give = u32::from_le_bytes(adler32buf.try_into()?);

                if  meta_adler32_give != meta_adler32_want {
                    return Err(elog!("[x] Want adler32 sum is {:#x} but given {:#x}", meta_adler32_want, meta_adler32_give));
                }

                let metabuf: Vec<u16> = metabuf.chunks_exact(2).map(|a| u16::from_le_bytes([a[0], a[1]])).collect();
                let metastr = String::from_utf16(&metabuf[..])
                    .context(elog!("failed to get metastr"))?
                    .replace("\x00", "");
                let metare = regex::Regex::new(r#"\s{1}(\w+)="(.*?)""#)
                    .context(elog!("regex error"))?;
                let mut meta = HashMap::new();
                for attr in metare.captures_iter(metastr.as_str()) {
                    meta.insert(attr[1].to_string(), attr[2].to_string());
                }

                Ok(meta)
            }
        ) >> ( meta )
    );
    let (buf, meta) = mdict_header?;
    println!("[+] Got header\n{:#x?}", meta);
    if let Some(option) = option {
        if option == ParseOption::OnlyHeader {
            return Ok(vec![]);
        }
    }

    let mdxinfo = &MDXInfo::new(meta)?;

    println!("[+] Parse words ...");
    // words: Vec<(word_text: String, meaning_offset: u64)>
    let words: NomResult<Vec<(String, u64)>> = do_parse!(buf,
        // layout: tuple(
        //     word_info_size: u64,
        //     word_block_size: u64,
        //     word_block_count: u64,
        //     word_count: u64,
        // )
        layout: map_res!(take!(if mdxinfo.version < 20 { 16usize } else { 44usize }),
            |x: &[u8]| -> AnyResult<(u64, u64, u64, u64)> {
                if mdxinfo.encid == 1 {
                    return Err(elog!("words layout is encrypted by the creator"));
                }

                let layoutbuf = if mdxinfo.version < 20 {
                    x
                } else {
                    let (layoutbuf, adler32buf) = x.split_at(x.len() - 4);
                    let mut adler = Adler32::new();
                    adler.write_slice(layoutbuf);
                    if adler.checksum() != u32::from_be_bytes(
                        adler32buf.try_into().context(elog!("convert to bytes failed"))?
                    ) {
                        return Err(elog!("wrong words layout adler32 checksum"));
                    }
                    layoutbuf
                };

                let layout: NomResult<_> = do_parse!(layoutbuf,
                    word_block_count: take!(mdxinfo.integersz) >>
                    word_count: take!(mdxinfo.integersz) >>
                    _word_info_unpack_size: cond!(mdxinfo.version >= 20, take!(mdxinfo.integersz)) >>
                    word_info_size: take!(mdxinfo.integersz) >>
                    word_block_size: take!(mdxinfo.integersz) >> (
                        (
                            bytes_to_u64(word_info_size, true),
                            bytes_to_u64(word_block_size, true),
                            bytes_to_u64(word_block_count, true),
                            bytes_to_u64(word_count, true),
                        )
                    )
                );

                let (_, layout) = layout.context(elog!("failed to get words layout"))?;
                Ok(layout)
            }
        ) >>
        // layout.0: word_info_size
        // infos: Vec<block_word_count: u64, packsz: u64, unpacksz: u64>
        infos: map_res!(take!(layout.0 as usize),
            |x: &[u8]| -> AnyResult<Vec<(u64, u64, u64)>> {
                let mut infosbuf = if mdxinfo.version < 20 {
                    x.to_vec()
                } else {
                    let r: NomResult<_> = tuple!(x, le_u32, take!(4));
                    let (data, (_packtype, adler32buf)) = r.context(elog!("take adler failed"))?;

                    let data = if mdxinfo.encid == 2 {
                        let ripemed128_message = [
                            adler32buf[0], adler32buf[1], adler32buf[2], adler32buf[3],
                            0x95, 0x36, 0x00, 0x00,
                        ];
                        let mut ripemd128 = Ripemd128::new();
                        ripemd128.input(ripemed128_message);
                        let ripemd128_key = ripemd128.result();
                        let cipher = x[8..].to_vec();
                        let decrypt_key = ripemd128_key.to_vec();
                        mdx_decrypt(cipher, decrypt_key).context(elog!("mdx_decrypt failed"))?
                    } else {
                        data.to_vec()
                    };

                    let mut infosbuf = vec![];
                    zlib::Decoder::new(&data[..])
                        .read_to_end(&mut infosbuf)
                        .context(elog!("zlib decoding failed"))?;
                    infosbuf
                };

                let mut infos = vec![];

                let get_word_size = |chrcnt| -> u64 {
                    if mdxinfo.version < 20 {
                        if mdxinfo.encoding == "UTF-16" {
                            chrcnt * 2
                        } else {
                            chrcnt
                        }
                    } else {
                        if mdxinfo.encoding == "UTF-16" {
                            (chrcnt + 1) * 2
                        } else {
                            chrcnt + 1
                        }
                    }
                };

                // layout.2: number of word block
                for i in 0..layout.2 {
                    let info: NomResult<_> = do_parse!(&infosbuf[..],
                        block_word_count: take!(mdxinfo.integersz) >>
                        first_word_size: take!(mdxinfo.integersz / 4) >>
                        _first_word: take!(get_word_size(bytes_to_u64(first_word_size, true))) >>
                        last_word_size: take!(mdxinfo.integersz / 4) >>
                        _last_word: take!(get_word_size(bytes_to_u64(last_word_size, true))) >>
                        packsz: take!(mdxinfo.integersz)>>
                        unpacksz: take!(mdxinfo.integersz) >>
                        (
                            {
                                (
                                    bytes_to_u64(block_word_count, true),
                                    bytes_to_u64(packsz, true),
                                    bytes_to_u64(unpacksz, true),
                                )
                            }
                        )
                    );
                    let (remain, info) = info.context(elog!("failed to parse mdx words info"))?;
                    println!("[+] block_word_count[{}] contians {} words", i, info.0);
                    infos.push(info);
                    infosbuf = remain.to_vec();
                }
                Ok(infos)
            } // lambda
        ) >>
        // layout.1: word_block_size
        words: map_res!(
            take!(layout.1 as usize),
            |mut x: &[u8]| -> AnyResult<Vec<(String, u64)>> {
                let mut words = vec![];
                for i in 0..infos.len() {
                    if x.len() == 0 {
                        break;
                    }

                    let (_block_word_count, packsz, unpacksz) = infos[i];
                    let packet = MdxPacket::new(x, packsz).context(elog!("mdxpacket error"))?;

                    let data = if packet.packtype == 1 {
                        minilzo_rs::LZO::init().context(elog!("failed to initialize minilzo"))?
                            .decompress_safe(&packet.data[..], unpacksz as usize)
                            .context(elog!("lzo decompress failed"))?
                    } else if packet.packtype == 2 {
                        let mut data = vec![];
                        zlib::Decoder::new(packet.data).read_to_end(&mut data)
                            .context(elog!("zlib decoding failed"))?;
                        let mut adler = Adler32::new();
                        adler.write_slice(&data[..]);
                        if adler.checksum() != packet.adler32 {
                            return Err(elog!("wrong word block {} adler32", i));
                        }
                        data
                    } else {
                        x.to_vec()
                    };

                    let mut data = &data[..];
                    let mut subwords: Vec<(String, u64)> = vec![];
                    loop {
                        if data.len() == 0 {
                            break;
                        }

                        let nullchar = if &mdxinfo.encoding == "UTF-16" {
                            "\x00\x00"
                        } else {
                            "\x00"
                        };

                        let r: NomResult<_> = do_parse!(data,
                            meaning_offset: take!(mdxinfo.integersz) >>
                            word_text: take_until!(nullchar) >>
                            _end: take!(nullchar.len()) >> (
                                (bytes_to_u64(meaning_offset, true), word_text)
                            )
                        );

                        let (remain, (meaning_offset, word_text)) = r
                            .context(elog!("meaning_offset"))?;
                        let word_text = mdx_decode(&mdxinfo, word_text)
                            .context(elog!(
                                "failed to decode {:x?} with encode {}",
                                word_text,
                                mdxinfo.encoding
                            ))?;
                        subwords.push((word_text, meaning_offset));
                        data = remain;
                    }
                    println!("[+] word_list[{}] contains {} words", i, subwords.len());
                    words.append(&mut subwords);

                    x = packet.remain;
                }
                Ok(words)
            }
        ) >>
        (
            words
        )
    ); // words parsing
    let (buf, words) = words.context(elog!("word parsing failed"))?;
    println!("[+] Got {} words", words.len());

    println!("[+] Parse meanings ...");
    let meanings: NomResult<Vec<u8>> = do_parse!(buf,
        meaning_block_count: take!(mdxinfo.integersz) >>
        word_count: take!(mdxinfo.integersz)  >>
        meaning_info_size: take!(mdxinfo.integersz) >>
        meaning_block_size: take!(mdxinfo.integersz) >>
        meanings: map_res!(
            tuple!(
                // meanings info of vector of (packsz, unpacksz) with length of meaning_block_count
                count!(
                    pair!(take!(mdxinfo.integersz), take!(mdxinfo.integersz)),
                    bytes_to_u64(meaning_block_count, true) as usize
                ),
                take!(bytes_to_u64(meaning_block_size, true))
            ),
            // Vec<(packsz, unpacksz)>, meaning_block
            |x: (Vec<(&[u8], &[u8])>, &[u8])| -> AnyResult<Vec<u8>> {
                let (infos, mut meaningsbuf) = x;
                let mut meanings: Vec<u8> = vec![];
                for (packsz, unpacksz) in infos {
                    let packsz = bytes_to_u64(packsz, true);
                    let unpacksz = bytes_to_u64(unpacksz, true);
                    let packet = MdxPacket::new(meaningsbuf, packsz as u64)
                        .context(elog!("failed to create MdxPacket"))?;
                    let mut unpackbuf = if mdxinfo.version < 20 {
                        minilzo_rs::LZO::init()
                            .context(elog!("failed to initialize lzo"))?
                            .decompress_safe(&packet.data[..], unpacksz as usize)
                            .context(elog!("meanings lzo decompress failed"))?
                    } else {
                        let mut unpackbuf: Vec<u8> = Vec::new();
                        zlib::Decoder::new(packet.data)
                            .read_to_end(&mut unpackbuf)
                            .context(elog!("zlib decoding failed"))?;

                        let mut adler = Adler32::new();
                        adler.write_slice(&unpackbuf[..]);
                        if adler.checksum() != packet.adler32 {
                            return Err(elog!("[x] wrong adler32 for meaning block"));
                        }
                        unpackbuf
                    };
                    meanings.append(&mut unpackbuf);
                    meaningsbuf = packet.remain;
                }
                Ok(meanings)
            }
        ) >>
        (
            meanings
        )
    );
    // ignore the remained buffer
    let (_, meanings) = meanings.context(elog!("failed to parse meaning "))?;

    let mut word_meaning_list: Vec<(String, String)> = vec![];
    println!("[+] Combine words and meanings ...");
    for i in 0..words.len() {
        let (start, word) = (words[i].1 as usize, words[i].0.clone());
        let end = if i == 0 {
            // the first element
            if words.len() == 1 {
                meanings.len()
            } else {
                words[1].1 as usize
            }
        } else if i == words.len() - 1{
            // the last element
            meanings.len()
        } else {
            // middle element
            words[i + 1].1 as usize
        };
        let meaning = mdx_decode(&mdxinfo, &meanings[start..end])
            .context(elog!(
                "failed to decode meaning {:x?} with encode {}",
                &meanings[start..end],
                mdxinfo.encoding
            ))?;
        word_meaning_list.push((word, meaning));
    }

    Ok(word_meaning_list)
}

pub fn create_mdx<P: AsRef<Path>>(srcpath: P, dstpath: P) -> AnyResult<()> {
    let dstpath = dstpath.as_ref();
    let mut dstmdx = File::create(dstpath).context(elog!("Cannot create {}", dstpath.display()))?;

    println!("[+] Write mdx header ...");
    let now: DateTime<Local> = Local::now();
    let mut meta = format!(
        r#"
			<Dictionary
			GeneratedByEngineVersion="{version}"
			RequiredEngineVersion="{version}"
			Encrypted="{encrypted}"
			Encoding="{encoding}"
			Format="html"
			CreationDate="{date}"
			Compact="No"
			Compat="No"
			KeyCaseSensitive="No"
			Description="{description}"
			Title="{title}"
			DataSourceFormat="106"
			StyleSheet=""
			RegisterBy="{creator}"
			RegCode="You do not need this"/>
        "#,
        version = "2.0",
        encrypted = 0,
        encoding = "UTF-8",
        date = now.format("%Y-%m-%d %H:%M:%S"),
        description = "MDX created by Wikit",
        title = "MDX 测试样例",
        creator = "The awesome author",
    );
    meta.push_str("\r\n\x00");
    let mut metabytes = vec![];
    for ch in meta.encode_utf16() {
        let bytes = ch.to_le_bytes();
        metabytes.push(bytes[0]);
        metabytes.push(bytes[1]);
    }

    let metasz = metabytes.len() as u32;

    let mut adler = Adler32::new();
    adler.write_slice(&metabytes[..]);
    let adler32 = adler.checksum() as u32;

    dstmdx.write(&metasz.to_be_bytes()[..])?;
    for ch in metabytes {
        dstmdx.write(&ch.to_be_bytes()[..])?;
    }
    dstmdx.write(&adler32.to_le_bytes()[..])?;

    println!("[+] Write mdx layout ...");
    let mut word_block_count = 0u64;
    let mut word_count = 0u64;
    let mut word_info_unpack_size = 0u64;
    let mut word_info_size = 0u64;
    let mut word_block_size = 0u64;

    let path = srcpath.as_ref();
    let file = File::open(path).context(elog!("Cannot open {:?}", path.display()))?;

    let reader = BufReader::new(file);
    let mut mdxsrc = MDXSource { iter: reader.lines() };

    Ok(())
}

pub async fn save_into_db(dict: Vec<(String, String)>, dburl: &str, table: &str) -> AnyResult<()> {
    let pool = PgPoolOptions::new().max_connections(5).connect(dburl).await?;
    sqlx::query(format!("CREATE TABLE IF NOT EXISTS {} (word TEXT UNIQUE, meaning TEXT)", table).as_str()).execute(&pool).await?;
    for word_meaing in dict {
        let (word, meaning) = word_meaing;
        sqlx::query(format!("INSERT INTO {} (word, meaning) VALUES ($1, $2) ON CONFLICT (word) DO NOTHING", table).as_str())
            .bind(word)
            .bind(meaning)
            .execute(&pool)
            .await?;
    }
    let rows: (i64, ) = sqlx::query_as(format!("SELECT COUNT(*) from {}", table).as_str())
        .fetch_one(&pool)
        .await?;
    println!("[+] The number of record in table {} is: {:?}", table, rows.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::mdict::parse_mdx;
    // this test is not a real unit-test but only for dirty and quick development
    #[test]
    fn test_parse_mdx() {
        let mdxpath = option_env!("TEST_MDX_FILE");
        if let Some(mdxpath) = mdxpath {
            let dict = parse_mdx(mdxpath, None);
            assert!(dict.is_ok(), "{}", "test mdx parsing failed");
        }
    }

    use crate::mdict::create_mdx;
    use std::path::Path;
    #[test]
    fn test_create_mdx() {
        let srcpath = Path::new("test/demo.txt");
        let dstpath = Path::new("test/demo.mdx");
        let r = create_mdx(srcpath, dstpath);
        assert!(r.is_ok(), "{}:{:?}", "create mdx failed", r);
    }
}
