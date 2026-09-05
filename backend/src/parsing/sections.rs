use super::text::paragraphs;

pub(super) struct Parts {
    pub body: Vec<String>,
    pub answer: Vec<String>,
    pub solution: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Default)]
struct Idx {
    ans: Option<usize>,
    sol: Option<usize>,
    note: Option<usize>,
}

pub(super) fn split(lines: &[&str], body_start: usize) -> Parts {
    let idx = find_sections(lines);
    let body_end = [idx.ans, idx.sol, idx.note]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(lines.len())
        .max(body_start);
    Parts {
        body: paragraphs(&lines[body_start..body_end]),
        answer: section_paragraphs(lines, idx.ans, &[idx.sol, idx.note]),
        solution: section_paragraphs(lines, idx.sol, &[idx.note]),
        notes: section_paragraphs(lines, idx.note, &[]),
    }
}

fn find_sections(lines: &[&str]) -> Idx {
    let find = |title: &str| lines.iter().position(|l| l.trim() == title);
    Idx {
        ans: find("## 答案"),
        sol: find("## 解析"),
        note: find("## 备注"),
    }
}

fn section_paragraphs(lines: &[&str], start: Option<usize>, ends: &[Option<usize>]) -> Vec<String> {
    let Some(a) = start else {
        return Vec::new();
    };
    let end = ends
        .iter()
        .flatten()
        .filter(|e| **e > a)
        .min()
        .copied()
        .unwrap_or(lines.len());
    paragraphs(&lines[a + 1..end])
}
