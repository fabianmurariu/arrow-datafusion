// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

extern crate criterion;

use arrow::array::{StringArray, StringViewArray};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use datafusion_expr::ColumnarValue;
use rand::distributions::Alphanumeric;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::Arc;

/// gen_arr(4096, 128, 0.1, 0.1, true) will generate a StringViewArray with
/// 4096 rows, each row containing a string with 128 random characters.
/// around 10% of the rows are null, around 10% of the rows are non-ASCII.
fn gen_string_array(
    n_rows: usize,
    str_len_chars: usize,
    null_density: f32,
    utf8_density: f32,
    is_string_view: bool, // false -> StringArray, true -> StringViewArray
) -> Vec<ColumnarValue> {
    let mut rng = StdRng::seed_from_u64(42);
    let rng_ref = &mut rng;

    let corpus = "DataFusionДатаФусион数据融合📊🔥"; // includes utf8 encoding with 1~4 bytes
    let corpus_char_count = corpus.chars().count();

    let mut output_string_vec: Vec<Option<String>> = Vec::with_capacity(n_rows);
    for _ in 0..n_rows {
        let rand_num = rng_ref.gen::<f32>(); // [0.0, 1.0)
        if rand_num < null_density {
            output_string_vec.push(None);
        } else if rand_num < null_density + utf8_density {
            // Generate random UTF8 string
            let mut generated_string = String::with_capacity(str_len_chars);
            for _ in 0..str_len_chars {
                let idx = rng_ref.gen_range(0..corpus_char_count);
                let char = corpus.chars().nth(idx).unwrap();
                generated_string.push(char);
            }
            output_string_vec.push(Some(generated_string));
        } else {
            // Generate random ASCII-only string
            let value = rng_ref
                .sample_iter(&Alphanumeric)
                .take(str_len_chars)
                .collect();
            let value = String::from_utf8(value).unwrap();
            output_string_vec.push(Some(value));
        }
    }

    if is_string_view {
        let string_view_array: StringViewArray = output_string_vec.into_iter().collect();
        vec![ColumnarValue::Array(Arc::new(string_view_array))]
    } else {
        let string_array: StringArray = output_string_vec.clone().into_iter().collect();
        vec![ColumnarValue::Array(Arc::new(string_array))]
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    // All benches are single batch run with 8192 rows
    let character_length = datafusion_functions::unicode::character_length();

    let n_rows = 8192;
    for str_len in [8, 32, 128, 4096] {
        // StringArray ASCII only
        let args_string_ascii = gen_string_array(n_rows, str_len, 0.1, 0.0, false);
        c.bench_function(
            &format!("character_length_StringArray_ascii_str_len_{}", str_len),
            |b| {
                b.iter(|| {
                    #[allow(deprecated)] // TODO use invoke_batch
                    black_box(character_length.invoke(&args_string_ascii))
                })
            },
        );

        // StringArray UTF8
        let args_string_utf8 = gen_string_array(n_rows, str_len, 0.1, 0.5, false);
        c.bench_function(
            &format!("character_length_StringArray_utf8_str_len_{}", str_len),
            |b| {
                b.iter(|| {
                    #[allow(deprecated)] // TODO use invoke_batch
                    black_box(character_length.invoke(&args_string_utf8))
                })
            },
        );

        // StringViewArray ASCII only
        let args_string_view_ascii = gen_string_array(n_rows, str_len, 0.1, 0.0, true);
        c.bench_function(
            &format!("character_length_StringViewArray_ascii_str_len_{}", str_len),
            |b| {
                b.iter(|| {
                    #[allow(deprecated)] // TODO use invoke_batch
                    black_box(character_length.invoke(&args_string_view_ascii))
                })
            },
        );

        // StringViewArray UTF8
        let args_string_view_utf8 = gen_string_array(n_rows, str_len, 0.1, 0.5, true);
        c.bench_function(
            &format!("character_length_StringViewArray_utf8_str_len_{}", str_len),
            |b| {
                b.iter(|| {
                    #[allow(deprecated)] // TODO use invoke_batch
                    black_box(character_length.invoke(&args_string_view_utf8))
                })
            },
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
