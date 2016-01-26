// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use rustc_serialize::json;
use std::collections::BTreeMap;
use rustc_serialize::base64::ToBase64;

use errors::FfiError;
use safe_nfs::file::File;
use std::sync::{Arc, Mutex};
use safe_core::client::Client;
use safe_nfs::helper::file_helper::FileHelper;
use safe_nfs::metadata::file_metadata::FileMetadata;

#[derive(Debug)]
pub struct GetFileResponse {
    content: String,
    metadata: Option<Metadata>,
}

#[derive(RustcEncodable, Debug)]
struct Metadata {
    name: String,
    size: i64,
    user_metadata: String,
    creation_time_sec: i64,
    creation_time_nsec: i64,
    modification_time_sec: i64,
    modification_time_nsec: i64,
}

pub fn get_response(file: &File,
                    client: Arc<Mutex<Client>>,
                    offset: i64,
                    length: i64,
                    include_metadata: bool)
                    -> Result<GetFileResponse, FfiError> {
    let file_metadata = if include_metadata {
        Some(get_file_metadata(file.get_metadata()))
    } else {
        None
    };

    let file_helper = FileHelper::new(client);
    let mut reader = file_helper.read(&file);
    let mut size = length as u64;
    if size == 0 {
        size = reader.size();
    };
    Ok(GetFileResponse {
        content: try!(reader.read(offset as u64, size)).to_base64(::config::get_base64_config()),
        metadata: file_metadata,
    })
}

fn get_file_metadata(file_metadata: &FileMetadata) -> Metadata {
    use rustc_serialize::base64::ToBase64;

    let created_time = file_metadata.get_created_time().to_timespec();
    let modified_time = file_metadata.get_modified_time().to_timespec();
    Metadata {
        name: file_metadata.get_name().clone(),
        size: file_metadata.get_size() as i64,
        user_metadata: (*file_metadata.get_user_metadata())
                           .to_base64(::config::get_base64_config()),
        creation_time_sec: created_time.sec,
        creation_time_nsec: created_time.nsec as i64,
        modification_time_sec: modified_time.sec,
        modification_time_nsec: modified_time.nsec as i64,
    }
}

impl json::ToJson for GetFileResponse {
    fn to_json(&self) -> json::Json {
        let mut response_tree = BTreeMap::new();
        let _ = response_tree.insert("content".to_string(), self.content.to_json());
        if let Some(ref metadata) = self.metadata {
            let json_metadata_str = unwrap_result!(json::encode(metadata));
            let _ = response_tree.insert("metadata".to_string(), json_metadata_str.to_json());
        }

        json::Json::Object(response_tree)
    }
}
