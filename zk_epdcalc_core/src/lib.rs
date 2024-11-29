/*
 * Copyright (c) 2024 Thomas Preindl
 * MIT License (see LICENSE or https://mit-license.org)
 */

pub trait VerifiedEpd<Epd> {
    fn get_zkp(&self) -> &str;
    fn get_epd(&self) -> &Epd;

    fn from_result(epd: Epd, zkp: Box<str>) -> Self;
}

