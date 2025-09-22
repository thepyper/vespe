use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{ParsedToolCall, StructuredOutputBlock, ToolCallPolicy};

// This file will contain the implementation of TaggedPolicy in Phase 2.
