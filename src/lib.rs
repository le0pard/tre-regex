use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

const MAX_NMATCH: usize = 10;
const REG_EXTENDED: c_int = 1;
const REG_ICASE: c_int = 2;

// --- Memory Layouts (Mirroring your Ruby FFI Structs) ---

#[repr(C)]
pub struct tre_regex_t {
  pub re_nsub: usize,
  pub value: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct regmatch_t {
  pub rm_so: c_int,
  pub rm_eo: c_int,
}

#[repr(C)]
pub struct tre_regaparams_t {
  pub cost_ins: c_int,
  pub cost_del: c_int,
  pub cost_subst: c_int,
  pub max_cost: c_int,
  pub max_ins: c_int,
  pub max_del: c_int,
  pub max_subst: c_int,
  pub max_err: c_int,
}

#[repr(C)]
pub struct tre_regamatch_t {
  pub nmatch: usize,
  pub pmatch: *mut regmatch_t,
  pub cost: c_int,
  pub num_ins: c_int,
  pub num_del: c_int,
  pub num_subst: c_int,
}

extern "C" {
  fn tre_regcomp(preg: *mut tre_regex_t, regex: *const c_char, cflags: c_int) -> c_int;
  fn tre_regfree(preg: *mut tre_regex_t);
  fn tre_reganexec(
    preg: *const tre_regex_t,
    string: *const c_char,
    len: usize,
    match_data: *mut tre_regamatch_t,
    params: tre_regaparams_t,
    eflags: c_int,
  ) -> c_int;
  fn tre_regaparams_default(params: *mut tre_regaparams_t);
}

// --- JS Objects ---

#[napi(object)]
pub struct TreRegexOptions {
  pub max_errors: Option<u32>,
  pub max_insertions: Option<u32>,
  pub max_deletions: Option<u32>,
  pub max_substitutions: Option<u32>,
  pub max_cost: Option<u32>,
  pub weight_insertion: Option<u32>,
  pub weight_deletion: Option<u32>,
  pub weight_substitution: Option<u32>,
}

#[napi(object)]
pub struct TreRegexErrors {
  pub insertions: u32,
  pub deletions: u32,
  pub substitutions: u32,
}

#[napi(object)]
pub struct TreRegexResult {
  pub match_text: String,
  pub submatches: Vec<Option<String>>,
  pub index: u32,
  pub end_index: u32,
  pub cost: u32,
  pub errors: TreRegexErrors,
}

// --- The Main Class ---

#[napi]
pub struct TreRegex {
  preg: *mut tre_regex_t,
}

#[napi]
impl TreRegex {
  #[napi(constructor)]
  pub fn new(pattern: String, ignore_case: Option<bool>) -> Result<Self> {
    let c_pattern =
      CString::new(pattern).map_err(|_| Error::from_reason("Pattern contains null bytes"))?;

    // Safely allocate the struct on the heap using Rust's Box
    let mut preg = Box::new(tre_regex_t {
      re_nsub: 0,
      value: std::ptr::null_mut(),
    });

    let mut flags = REG_EXTENDED;
    if ignore_case.unwrap_or(false) {
      flags |= REG_ICASE;
    }

    // Pass a mutable reference to the boxed memory
    let res = unsafe { tre_regcomp(&mut *preg, c_pattern.as_ptr(), flags) };
    if res != 0 {
      return Err(Error::from_reason("Failed to compile regex pattern"));
    }

    // Consume the Box so Rust doesn't drop the memory right now!
    Ok(Self {
      preg: Box::into_raw(preg),
    })
  }

  #[napi]
  pub fn test(&self, text: String, options: Option<TreRegexOptions>) -> bool {
    self.exec(text, options).is_some()
  }

  #[napi]
  pub fn exec(&self, text: String, options: Option<TreRegexOptions>) -> Option<TreRegexResult> {
    let (payload, _, _) = self.execute_and_extract(&text, 0, 0, &options)?;
    Some(payload)
  }

  #[napi]
  pub fn match_all(&self, text: String, options: Option<TreRegexOptions>) -> Vec<TreRegexResult> {
    let mut results = Vec::new();
    let mut b_off = 0;
    let mut c_off = 0;
    let byte_len = text.len();

    while b_off <= byte_len {
      if let Some((payload, adv_b, adv_c)) = self.execute_and_extract(&text, b_off, c_off, &options)
      {
        results.push(payload);

        let mut actual_adv_b = adv_b;
        let mut actual_adv_c = adv_c;

        // Zero-width infinite loop protection
        if actual_adv_b == 0 {
          if b_off == byte_len {
            break;
          }
          let next_char = text[b_off..].chars().next().unwrap();
          actual_adv_b = next_char.len_utf8();
          actual_adv_c = 1;
        }

        b_off += actual_adv_b;
        c_off += actual_adv_c;
      } else {
        break;
      }
    }
    results
  }

  // --- Private Extract Logic ---

  fn execute_and_extract(
    &self,
    full_text: &str,
    byte_off: usize,
    char_off: u32,
    options: &Option<TreRegexOptions>,
  ) -> Option<(TreRegexResult, usize, u32)> {
    let params = self.build_params(options);

    let mut pmatch = [regmatch_t { rm_so: 0, rm_eo: 0 }; MAX_NMATCH];
    let mut amatch = tre_regamatch_t {
      nmatch: MAX_NMATCH,
      pmatch: pmatch.as_mut_ptr(),
      cost: 0,
      num_ins: 0,
      num_del: 0,
      num_subst: 0,
    };

    let slice_to_search = &full_text[byte_off..];

    let res = unsafe {
      tre_reganexec(
        self.preg,
        slice_to_search.as_ptr() as *const c_char,
        slice_to_search.len(),
        &mut amatch,
        params,
        0,
      )
    };

    if res != 0 {
      return None;
    }

    let rm_so = pmatch[0].rm_so as usize;
    let rm_eo = pmatch[0].rm_eo as usize;

    // Get the raw byte array so we can slice it safely using C byte offsets
    let slice_bytes = slice_to_search.as_bytes();

    // 1. Zero-cost prefix extraction
    let prefix_str = unsafe { std::str::from_utf8_unchecked(&slice_bytes[..rm_so]) };
    let prefix_chars = prefix_str.encode_utf16().count() as u32;
    let start_char_index = char_off + prefix_chars;

    // 2. Zero-cost match text extraction
    let match_str = unsafe { std::str::from_utf8_unchecked(&slice_bytes[rm_so..rm_eo]) };
    let match_chars = match_str.encode_utf16().count() as u32;
    let end_char_index = start_char_index + match_chars;

    // 3. Zero-cost Capture Groups
    let mut submatches: Vec<Option<String>> = (1..MAX_NMATCH)
      .map(|i| {
        let so = pmatch[i].rm_so;
        let eo = pmatch[i].rm_eo;
        if so == -1 {
          None
        } else {
          let sub_bytes = &slice_bytes[so as usize..eo as usize];
          Some(unsafe { std::str::from_utf8_unchecked(sub_bytes) }.to_owned())
        }
      })
      .collect();

    // Cleanup trailing Nones
    while let Some(None) = submatches.last() {
      submatches.pop();
    }

    let payload = TreRegexResult {
      match_text: match_str.to_owned(),
      submatches,
      index: start_char_index,
      end_index: end_char_index,
      cost: amatch.cost as u32,
      errors: TreRegexErrors {
        insertions: amatch.num_ins as u32,
        deletions: amatch.num_del as u32,
        substitutions: amatch.num_subst as u32,
      },
    };

    Some((payload, rm_eo, prefix_chars + match_chars))
  }

  fn build_params(&self, opts: &Option<TreRegexOptions>) -> tre_regaparams_t {
    let mut params: tre_regaparams_t = unsafe { std::mem::zeroed() };
    unsafe { tre_regaparams_default(&mut params) };

    let mut has_max_errors = false;
    let mut has_max_cost = false;

    if let Some(o) = opts {
      if let Some(e) = o.max_errors {
        params.max_err = e as c_int;
        has_max_errors = true;
      }
      if let Some(c) = o.max_cost {
        params.max_cost = c as c_int;
        has_max_cost = true;
      }

      // Mimic Ruby: if max_errors isn't provided, strictly disable specific errors by default
      if let Some(i) = o.max_insertions {
        params.max_ins = i as c_int;
      } else if !has_max_errors {
        params.max_ins = 0;
      }

      if let Some(d) = o.max_deletions {
        params.max_del = d as c_int;
      } else if !has_max_errors {
        params.max_del = 0;
      }

      if let Some(s) = o.max_substitutions {
        params.max_subst = s as c_int;
      } else if !has_max_errors {
        params.max_subst = 0;
      }

      if let Some(w) = o.weight_insertion {
        params.cost_ins = w as c_int;
      }
      if let Some(w) = o.weight_deletion {
        params.cost_del = w as c_int;
      }
      if let Some(w) = o.weight_substitution {
        params.cost_subst = w as c_int;
      }
    } else {
      // Force exact matching if no options object is provided at all
      params.max_ins = 0;
      params.max_del = 0;
      params.max_subst = 0;
    }

    // Calculate total max_err if only granular limits (like maxSubstitutions: 1) were provided
    if !has_max_errors && !has_max_cost {
      params.max_err = params.max_ins + params.max_del + params.max_subst;
    }

    params
  }
}

// --- Flawless Memory Cleanup ---
impl Drop for TreRegex {
  fn drop(&mut self) {
    if !self.preg.is_null() {
      unsafe {
        tre_regfree(self.preg);
        // "Un-box" the raw pointer so Rust's garbage collector safely destroys it
        let _ = Box::from_raw(self.preg);
      }
    }
  }
}
