//! Complete authored source blocks for native capture and coverage.
//!
//! A block is separated by a blank line, not inferred from punctuation. Keeping
//! its internal lines intact prevents admission from dropping an adjacent
//! condition. This is a source-integrity boundary, not a semantic classifier:
//! dependencies between blocks and whether a block is durable still need review.

pub const SOURCE_BLOCKS_MAX: usize = 64;

/// Borrow complete, trimmed blocks without normalizing their internal text.
pub fn source_blocks(task: &str) -> SourceBlocks<'_> {
    SourceBlocks { remaining: task }
}

pub struct SourceBlocks<'a> {
    remaining: &'a str,
}

impl<'a> Iterator for SourceBlocks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut start = None;
        let mut consumed = 0;
        for line in self.remaining.split_inclusive('\n') {
            let line_start = consumed;
            consumed += line.len();
            if line.trim().is_empty() {
                if let Some(start) = start {
                    let block = self.remaining[start..line_start].trim();
                    self.remaining = &self.remaining[consumed..];
                    return Some(block);
                }
            } else if start.is_none() {
                start = Some(line_start);
            }
        }
        let remaining = std::mem::take(&mut self.remaining);
        start.map(|start| remaining[start..].trim())
    }
}

/// Accept one unambiguous passage spanning complete contiguous source blocks.
/// Cropped boundaries, ambiguous repeated passages and over-limit tasks have no proof.
pub fn unique_source_passage(task: &str, content: &str) -> Option<std::ops::Range<usize>> {
    if content.is_empty() {
        return None;
    }
    let mut bounds = [(0, 0); SOURCE_BLOCKS_MAX];
    let mut count = 0;
    let origin = task.as_ptr() as usize;
    for (index, block) in source_blocks(task).enumerate() {
        if index >= SOURCE_BLOCKS_MAX {
            return None;
        }
        let start = block.as_ptr() as usize - origin;
        bounds[index] = (start, start + block.len());
        count = index + 1;
    }
    let bounds = &bounds[..count];
    let mut found = None;
    for (first, &(start, _)) in bounds.iter().enumerate() {
        let end = start.checked_add(content.len())?;
        if let Ok(last) = bounds.binary_search_by_key(&end, |&(_, end)| end) {
            if last >= first && task.get(start..end) == Some(content) {
                if found.is_some() {
                    return None;
                }
                found = Some(first..last + 1);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_blocks_preserve_conditions_and_original_internal_line_endings() {
        let task = "\r\n  Use SQLite for integration tests.\r\nOnly isolated tests; production uses PostgreSQL.  \r\n\t\r\n\r\n  Keep café data local.\n\n";
        assert_eq!(
            source_blocks(task).collect::<Vec<_>>(),
            [
                "Use SQLite for integration tests.\r\nOnly isolated tests; production uses PostgreSQL.",
                "Keep café data local.",
            ]
        );
    }

    #[test]
    fn admission_refuses_cropped_or_normalized_quotes() {
        let task =
            "Use SQLite for integration tests.\nOnly isolated tests; production uses PostgreSQL.";
        assert_eq!(unique_source_passage(task, task), Some(0..1));
        assert_eq!(
            unique_source_passage(task, "Use SQLite for integration tests."),
            None
        );
        assert_eq!(unique_source_passage(task, &task.replace('\n', " ")), None);
    }

    #[test]
    fn repeated_source_text_is_ambiguous() {
        let task = "Never upload raw tasks.\n\nNever upload raw tasks.";
        assert_eq!(unique_source_passage(task, "Never upload raw tasks."), None);
        assert_eq!(unique_source_passage("\n \n", ""), None);
    }

    #[test]
    fn a_quote_cannot_bypass_the_source_block_limit() {
        let allowed = (0..SOURCE_BLOCKS_MAX)
            .map(|index| format!("Policy {index}."))
            .collect::<Vec<_>>()
            .join("\n\n");
        assert_eq!(unique_source_passage(&allowed, "Policy 0."), Some(0..1));
        let oversized = format!("{allowed}\n\nAnother policy.");
        assert_eq!(unique_source_passage(&oversized, "Policy 0."), None);
    }

    #[test]
    fn linked_passages_keep_cross_block_context_and_refuse_cropped_boundaries() {
        let context = "This exception applies only to recovery drills.";
        let policy = "Operators may disable tracing during restoration.";
        let task = format!("Routine request.\n\n{context}\n\n{policy}\n\nAnother request.");
        let passage = format!("{context}\n\n{policy}");
        assert_eq!(unique_source_passage(&task, &passage), Some(1..3));
        assert_eq!(
            unique_source_passage(&task, &passage[..passage.len() - 1]),
            None
        );
        let repeated = "Policy A.\n\nPolicy A.\n\nPolicy A.";
        assert_eq!(
            unique_source_passage(repeated, "Policy A.\n\nPolicy A."),
            None
        );
    }
}
