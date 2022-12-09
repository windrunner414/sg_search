use std::cmp::Ordering;

#[inline]
pub fn calc_idf(df: u64, total_doc_num: u64) -> f64 {
    1f64 + f64::ln(total_doc_num as f64 / (df + 1) as f64)
}

pub trait TermPriorityCalculator {
    fn calc(&self, df: u64, tf: u16) -> f64;
}

#[derive(Debug)]
pub struct TfIdfTermPriorityCalculator {
    total_doc_num: u64,
}

impl TfIdfTermPriorityCalculator {
    #[inline]
    pub fn new(total_doc_num: u64) -> Self {
        TfIdfTermPriorityCalculator { total_doc_num }
    }
}

impl TermPriorityCalculator for TfIdfTermPriorityCalculator {
    #[inline]
    fn calc(&self, df: u64, tf: u16) -> f64 {
        calc_idf(df, self.total_doc_num) * tf as f64
    }
}

#[inline]
pub unsafe fn calc_cosine_unchecked(a: &[f64], b: &[f64]) -> f64 {
    let (mut product, mut q_sum_a, mut q_sum_b) = (0f64, 0f64, 0f64);

    for i in 0..a.len() {
        let an = a.get_unchecked(i);
        let bn = b.get_unchecked(i);
        product += an * bn;
        q_sum_a += an * an;
        q_sum_b += bn * bn;
    }

    product / (q_sum_a.sqrt() * q_sum_b.sqrt())
}

#[derive(Debug)]
pub struct Score {
    cosine: f64,
}

impl Score {
    pub fn new(a: &[f64], b: &[f64]) -> Self {
        Score {
            cosine: unsafe { calc_cosine_unchecked(a, b) },
        }
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.cosine.eq(&other.cosine)
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cosine.partial_cmp(&other.cosine)
    }
}

impl Eq for Score {}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.cosine > other.cosine {
            Ordering::Greater
        } else if self.cosine < other.cosine {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}
