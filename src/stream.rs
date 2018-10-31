// Copied from actix-web, with minor modifications. Actix Copyright (c) 2017 Nikolay Kim
// Original source: https://github.com/actix/actix-web/blob/v0.7.8/src/fs.rs (Commit hash 1716380f0890a1e936d84181effeb63906c1e609)
extern crate actix_web;

use actix_web::{HttpRequest, http::header};

pub fn calculate_ranges(req: &HttpRequest, length: u64) -> (u64, u64) {
	if let Some(ranges) = req.headers().get(header::RANGE) {
		if let Ok(rangesheader) = ranges.to_str() {
			if let Ok(rangesvec) = HttpRange::parse(rangesheader, length) {
				return (rangesvec[0].length, rangesvec[0].start)
			} else {
				return (length, 0);
			};
		} else {
			return (length, 0);
		};
	};
	return (length, 0);
}

#[derive(Debug, Clone, Copy)]
pub struct HttpRange {
    pub start: u64,
    pub length: u64,
}

static PREFIX: &'static str = "bytes=";
const PREFIX_LEN: usize = 6;

impl HttpRange {
    pub fn parse(header: &str, size: u64) -> Result<Vec<HttpRange>, ()> {
        if header.is_empty() {
            return Ok(Vec::new());
        }
        if !header.starts_with(PREFIX) {
            return Err(());
        }

        let size_sig = size as i64;
        let mut no_overlap = false;

        let all_ranges: Vec<Option<HttpRange>> = header[PREFIX_LEN..]
            .split(',')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|ra| {
                let mut start_end_iter = ra.split('-');

                let start_str = start_end_iter.next().ok_or(())?.trim();
                let end_str = start_end_iter.next().ok_or(())?.trim();

                if start_str.is_empty() {
                    let mut length: i64 = try!(end_str.parse().map_err(|_| ()));

                    if length > size_sig {
                        length = size_sig;
                    }

                    Ok(Some(HttpRange {
                        start: (size_sig - length) as u64,
                        length: length as u64,
                    }))
                } else {
                    let start: i64 = start_str.parse().map_err(|_| ())?;

                    if start < 0 {
                        return Err(());
                    }
                    if start >= size_sig {
                        no_overlap = true;
                        return Ok(None);
                    }

                    let length = if end_str.is_empty() {
                        size_sig - start
                    } else {
                        let mut end: i64 = end_str.parse().map_err(|_| ())?;

                        if start > end {
                            return Err(());
                        }

                        if end >= size_sig {
                            end = size_sig - 1;
                        }

                        end - start + 1
                    };

                    Ok(Some(HttpRange {
                        start: start as u64,
                        length: length as u64,
                    }))
                }
            }).collect::<Result<_, _>>()?;

        let ranges: Vec<HttpRange> = all_ranges.into_iter().filter_map(|x| x).collect();

        if no_overlap && ranges.is_empty() {
            return Err(());
        }

        Ok(ranges)
    }
}
