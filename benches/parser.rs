#![feature(test)]
extern crate test;

use pica::parse_subfield;
use rand::distributions::Alphanumeric;
use rand::{prelude::*, thread_rng, Rng};
use std::iter;
use test::Bencher;

const SUBFIELD_CODES: &'static [(char, usize)] = &[
    ('0', 432927591),
    ('1', 1),
    ('2', 1089021),
    ('3', 405116),
    ('4', 58588390),
    ('5', 12777937),
    ('6', 9570948),
    ('7', 47220747),
    ('8', 1),
    ('9', 61040098),
    ('A', 47482252),
    ('B', 35182779),
    ('C', 6056),
    ('D', 31520759),
    ('E', 36151752),
    ('F', 6473124),
    ('G', 8480732),
    ('H', 34512276),
    ('I', 3505993),
    ('J', 3499728),
    ('K', 13543811),
    ('L', 10002124),
    ('M', 1642543),
    ('N', 997916),
    ('O', 2261428),
    ('P', 1208748),
    ('Q', 17876),
    ('R', 1009775),
    ('S', 42771647),
    ('T', 2989000),
    ('U', 655379),
    ('V', 45910126),
    ('W', 1),
    ('X', 462524),
    ('Y', 14105725),
    ('Z', 307357),
    ('a', 581887302),
    ('b', 161163685),
    ('c', 23248798),
    ('d', 64692136),
    ('e', 55166475),
    ('f', 22337962),
    ('g', 30855211),
    ('h', 23680246),
    ('i', 10806443),
    ('j', 8140981),
    ('k', 386564),
    ('l', 7050324),
    ('m', 16312362),
    ('n', 19291292),
    ('o', 3088909),
    ('p', 21085059),
    ('q', 5629049),
    ('r', 13825878),
    ('s', 1067977),
    ('t', 32500874),
    ('u', 10295914),
    ('v', 9840205),
    ('w', 292),
    ('x', 9172615),
    ('y', 2915911),
    ('z', 2384726),
];

#[bench]
fn bench_parse_subfield(b: &mut Bencher) {
    let mut rng = thread_rng();

    b.iter(|| {
        let code = SUBFIELD_CODES
            .choose_weighted(&mut rng, |item| item.1)
            .unwrap()
            .0;
        let value: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(23)
            .collect();
        let input = format!("\u{1f}{}{}", code, value);
        assert!(parse_subfield(&input).is_ok())
    })
}
