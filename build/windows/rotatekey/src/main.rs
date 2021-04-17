// This file is part of Nuchain.

// Copyright (C) 2021 Rantai Nusantara Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Stand-alone program to easily rotate key (used commonly in Windows).

use std::process::Command;
use std::fs;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let home_dir = dirs::home_dir().expect("Cannot get working directory");

    let body = client
        .post("http://localhost:9933")
        .header("Content-Type", "application/json")
        .body(
            r#"
            {"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}
            "#,
        )
        .send()
        .await.expect("Cannot connect to local node")
        .text()
        .await
        .expect("Cannot get response from local node");

    // file.write_all(body).unwrap();
    let data = format!("{}", body);
    println!("Node Resp: {}", data);
    let data = {
        let s:Vec<&str> = data.split(":").collect::<Vec<_>>();
        format!("SESSION KEY:\n\r{}", &s[2][1..(&s[2]).len()-6])
    };
    let out_path = home_dir.join("ROTATEKEY.txt");
    fs::write(out_path.as_path(), format!("{}", data)).expect("Cannot write ROTATEKEY.txt");

    let output = Command::new("notepad")
        .arg(out_path)
        .output()
        .expect("Failed to execute command");

    println!("{:?}", output);
}
