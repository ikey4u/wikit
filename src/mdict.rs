use crate::elog;
use crate::error::{Context, AnyResult, WikitError};

use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Read;
use std::fs::File;

use sqlx::postgres::PgPoolOptions;
use nom::number::streaming::{be_u16, be_u32, be_u64, le_u32};
use nom::{regex, do_parse, tuple, map_res, take, count, take_till, pair};
use compress::zlib;
use adler::Adler32;
use ripemd128::{Ripemd128, Digest};

type NomResult<'a, O> = AnyResult<(&'a [u8], O), nom::Err<WikitError>>;

#[derive(PartialEq)]
pub enum ParseOption {
    OnlyHeader,
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

    // Integer width in MDX file
    let _integersz = if let Some(version) = meta.get("GeneratedByEngineVersion") {
        if version == "1.2" {
            4
        } else {
            8
        }
    } else {
        return Err(elog!("[!] No engine version"));
    };

    let encrypt_id = if let Some(encrypt_id) = meta.get("Encrypted") {
        match encrypt_id.as_str() {
            "Yes" | "yes" => 1,
            "No" | "no" => 0,
            _ => {
                if let Ok(encrypt_id) = encrypt_id.parse::<u32>() {
                    encrypt_id
                } else {
                    0
                }
            }
        }
    } else {
        0
    };

    println!("[+] Parse words ...");
    // words: Vec<(word_text: String, meaning_offset: u64)>
    let words: NomResult<Vec<(String, u64)>> = do_parse!(buf,
        // layout: tuple(
        //     word_info_size: u64,
        //     word_block_size: u64,
        //     word_block_count: u64,
        //     word_count: u64,
        //     word_info_unpack_size: u64
        // )
        layout: map_res!(take!(44usize),
            |x: &[u8]| -> AnyResult<(u64, u64, u64, u64, u64)> {
                if encrypt_id == 1 {
                    return Err(elog!("key_header which is encrypted by the creator"));
                }

                let (layoutbuf, adler32buf) = x.split_at(x.len() - 4);

                let mut adler = Adler32::new();
                adler.write_slice(layoutbuf);
                if adler.checksum() != u32::from_be_bytes(
                    adler32buf.try_into().context(elog!("convert to bytes failed"))?
                ) {
                    return Err(elog!("wrong word layout adler32 checksum"));
                }
                let layout: NomResult<_> = do_parse!(layoutbuf,
                    word_block_count: be_u64 >>
                    word_count: be_u64 >>
                    word_info_unpack_size: be_u64 >>
                    word_info_size: be_u64 >>
                    word_block_size: be_u64 >> (
                        (
                            word_info_size,
                            word_block_size,
                            word_block_count,
                            word_count,
                            word_info_unpack_size,
                        )
                    )
                );
                let (_, layout) = layout.context(elog!("failed to get layout"))?;
                println!("word count: {}", layout.3);
                Ok(layout)
            }
        ) >>
        // infos: Vec<block_word_count: u64, packsz: u64, unpacksz: u64>
        infos: map_res!(take!(layout.0 as usize),
            |x: &[u8]| -> AnyResult<Vec<(u64, u64, u64)>> {
                let r: NomResult<_> = tuple!(x, le_u32, take!(4));
                let (data, (packtype, adler32buf)) = r.context(elog!("take adler failed"))?;

                if packtype != 2 {
                    return Err(elog!("[x] Only support zlib compression"));
                }

                let data = if encrypt_id == 2 {
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

                let mut infos = vec![];
                // layout.2: number of word block
                for i in 0..layout.2 {
                    let info: NomResult<_> = do_parse!(&infosbuf[..],
                        block_word_count: be_u64 >>
                        first_word_size: be_u16 >>
                        first_word: take!(first_word_size + 1) >>
                        last_word_size: be_u16 >>
                        last_word: take!(last_word_size + 1) >>
                        packsz: be_u64 >>
                        unpacksz: be_u64 >>
                        (
                            {
                                let _first_word = String::from_utf8(first_word.to_vec())
                                    .context(elog!("parse first_word error"))?
                                    .replace("\x00", "");

                                let _last_word = String::from_utf8(last_word.to_vec())
                                    .context(elog!("parse last_word error"))?
                                    .replace("\x00", "");

                                println!("block_word_count[{}] contians {} words", i, block_word_count);

                                (
                                    block_word_count,
                                    packsz,
                                    unpacksz,
                                )
                            }
                        )
                    );

                    let (remain, info) = info.context(elog!("get remain failed"))?;
                    infos.push(info);
                    infosbuf = remain.to_vec();
                } // loop
                Ok(infos)
            } // lambda
        ) >>
        words: map_res!(
            take!(layout.1 as usize),
            |mut x: &[u8]| -> AnyResult<Vec<(String, u64)>> {
                let mut words = vec![];
                for i in 0..infos.len() {
                    if x.len() == 0 {
                        break;
                    }

                    let (_block_word_count, packsz, _unpacksz) = infos[i];
                    let packet = MdxPacket::new(x, packsz).context(elog!("mdxpacket error"))?;

                    if packet.packtype != 2 {
                        return Err(elog!("[x] Only support zlib compression"));
                    }

                    let mut data = Vec::new();
                    zlib::Decoder::new(packet.data).read_to_end(&mut data)
                        .context(elog!("zlib decoding failed"))?;
                    let mut data = &data[..];

                    let mut adler = Adler32::new();
                    adler.write_slice(data);
                    if adler.checksum() != packet.adler32 {
                        return Err(elog!("wrong word block {} adler32", i));
                    }

                    let mut subwords: Vec<(String, u64)> = vec![];
                    loop {
                        if data.len() == 0 {
                            break;
                        }

                        let r: NomResult<_> = tuple!(data,
                            be_u64,
                            take_till!(|x: u8 | -> bool { x == 0x00 }),
                            take!(1)
                        );
                        let (remain, (meaning_offset, word_text, _null_char)) = r.context(elog!("meaning_offset"))?;
                        let word_text = String::from_utf8(word_text.to_vec())
                            .context(elog!("invalid utf8 word text"))?
                            .replace("\x00", "");
                        subwords.push((word_text, meaning_offset));
                        data = remain;
                    }
                    println!("word_list[{}] contains {} words", i, subwords.len());
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
        meaning_block_count: be_u64 >>
        word_count: be_u64 >>
        meaning_info_size: be_u64 >>
        meaning_block_size: be_u64 >>
        meanings: map_res!(
            tuple!(
                // packsz and unpacksz
                count!(pair!(be_u64, be_u64), meaning_block_count as usize),
                take!(meaning_block_size)
            ),
            |x: (Vec<(u64, u64)>, &[u8])| -> AnyResult<Vec<u8>> {
                let (infos, mut meaningsbuf) = x;
                let mut meanings: Vec<u8> = vec![];
                for (packsz, _unpacksz) in infos {
                    let packet = MdxPacket::new(meaningsbuf, packsz)
                        .context(elog!("failed to create MdxPacket"))?;
                    if packet.packtype != 2 {
                        return Err(elog!("[x] Only support zlib compression"));
                    }

                    let mut unpackbuf: Vec<u8> = Vec::new();
                    zlib::Decoder::new(packet.data)
                        .read_to_end(&mut unpackbuf)
                        .context(elog!("zlib decoding failed"))?;

                    let mut adler = Adler32::new();
                    adler.write_slice(&unpackbuf[..]);
                    if adler.checksum() != packet.adler32 {
                        return Err(elog!("[x] wrong adler32 for meaning block"));
                    }

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
        let meaning = String::from_utf8(meanings[start..end].to_vec())
            .context(elog!("failed to get string from {:?}", &meanings[start..end]))?
            .replace("\x00", "");
        word_meaning_list.push((word, meaning));
    }

    Ok(word_meaning_list)
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
        let mdxpath = env!("TEST_MDX_FILE", "You must supply TEST_MDX_FILE environment");
        if let Err(e) = parse_mdx(mdxpath, None) {
            println!("failed to parse mdx: {:?}", e);
        }
    }
}
