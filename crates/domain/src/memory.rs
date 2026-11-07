//! Knowledge and memory context: scoped documents, chunking, hybrid retrieval,
//! deduplication, secret quarantine and compaction policy.

use chrono::{DateTime, Utc};

use crate::error::DomainError;
use crate::ids::{AgentId, ChunkId, DocumentId, LabId, ProjectId};
use crate::value::{ContentHash, Version};

/// Memory scope. There is no global scope in v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryScope {
    /// Lab-wide knowledge.
    Lab,
    /// Project knowledge.
    Project,
    /// Agent knowledge.
    Agent,
}

impl MemoryScope {
    /// Scope boost used by the hybrid score.
    #[must_use]
    pub const fn boost(self) -> f64 {
        match self {
            Self::Project => 1.0,
            Self::Lab => 0.8,
            Self::Agent => 0.9,
        }
    }
}

/// Document kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    /// SOW document.
    Sow,
    /// Plan document.
    Plan,
    /// Architecture decision.
    Adr,
    /// Note.
    Note,
    /// Transcript.
    Transcript,
    /// Artifact.
    Artifact,
    /// Policy.
    Policy,
    /// Skill.
    Skill,
}

/// Target embedding dimension.
pub const EMBEDDING_DIM: usize = 384;
/// Chunk target size in words.
pub const CHUNK_WORDS: usize = 800;
/// Chunk overlap in words (15 percent).
pub const CHUNK_OVERLAP_WORDS: usize = 120;
/// Near-duplicate cosine threshold.
pub const DEDUPE_COSINE: f64 = 0.92;

/// A chunk with its embedding.
#[derive(Debug, Clone)]
pub struct MemoryChunk {
    id: ChunkId,
    ordinal: u32,
    text: String,
    embedding: Vec<f32>,
    token_count: u32,
}

impl MemoryChunk {
    /// Build a chunk, validating dimension and token count.
    pub fn new(
        id: ChunkId,
        ordinal: u32,
        text: impl Into<String>,
        embedding: Vec<f32>,
        token_count: u32,
    ) -> Result<Self, DomainError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(DomainError::validation("chunk.text", "must not be empty"));
        }
        if embedding.len() != EMBEDDING_DIM {
            return Err(DomainError::validation(
                "chunk.embedding",
                "must have 384 dimensions",
            ));
        }
        if token_count == 0 {
            return Err(DomainError::validation(
                "chunk.token_count",
                "must be positive",
            ));
        }
        Ok(Self {
            id,
            ordinal,
            text,
            embedding,
            token_count,
        })
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> ChunkId {
        self.id
    }

    /// Ordinal within the document.
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    /// Chunk text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Embedding.
    #[must_use]
    pub fn embedding(&self) -> &[f32] {
        &self.embedding
    }

    /// Token count.
    #[must_use]
    pub const fn token_count(&self) -> u32 {
        self.token_count
    }
}

/// A scoped memory document with its chunks.
#[derive(Debug, Clone)]
pub struct MemoryDocument {
    id: DocumentId,
    lab_id: LabId,
    scope: MemoryScope,
    project_id: Option<ProjectId>,
    agent_id: Option<AgentId>,
    kind: MemoryKind,
    title: String,
    content: String,
    content_hash: ContentHash,
    version: Version,
    created_at: DateTime<Utc>,
    chunks: Vec<MemoryChunk>,
}

impl MemoryDocument {
    /// Build a document, quarantining secrets and chunking deterministically.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: DocumentId,
        lab_id: LabId,
        scope: MemoryScope,
        project_id: Option<ProjectId>,
        agent_id: Option<AgentId>,
        kind: MemoryKind,
        title: impl Into<String>,
        content: impl Into<String>,
        embedding_for: &dyn Fn(&str) -> Vec<f32>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        validate_scope(scope, project_id, agent_id)?;
        let title = title.into();
        if title.trim().is_empty() {
            return Err(DomainError::validation(
                "document.title",
                "must not be empty",
            ));
        }
        let content = content.into();
        if content.trim().is_empty() {
            return Err(DomainError::validation(
                "document.content",
                "must not be empty",
            ));
        }
        if contains_secret(&content) {
            return Err(DomainError::policy(
                "MEMORY_QUARANTINED",
                "document contains secret material and was not embedded",
            ));
        }
        let content_hash = ContentHash::from_bytes(content.as_bytes());
        let chunks = chunk_text(&content)
            .into_iter()
            .enumerate()
            .map(|(index, text)| {
                let words = text.split_whitespace().count();
                MemoryChunk::new(
                    ChunkId::generate(),
                    u32::try_from(index).unwrap_or(u32::MAX),
                    text,
                    embedding_for(&content),
                    u32::try_from(words).unwrap_or(u32::MAX),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id,
            lab_id,
            scope,
            project_id,
            agent_id,
            kind,
            title,
            content,
            content_hash,
            version: Version::ONE,
            created_at,
            chunks,
        })
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> DocumentId {
        self.id
    }

    /// Lab scope.
    #[must_use]
    pub const fn lab_id(&self) -> LabId {
        self.lab_id
    }

    /// Scope kind.
    #[must_use]
    pub const fn scope(&self) -> MemoryScope {
        self.scope
    }

    /// Project scope id.
    #[must_use]
    pub const fn project_id(&self) -> Option<ProjectId> {
        self.project_id
    }

    /// Agent scope id.
    #[must_use]
    pub const fn agent_id(&self) -> Option<AgentId> {
        self.agent_id
    }

    /// Document kind.
    #[must_use]
    pub const fn kind(&self) -> MemoryKind {
        self.kind
    }

    /// Title.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Raw content.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Aggregate version.
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Content hash.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }

    /// Creation time.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Chunks.
    #[must_use]
    pub fn chunks(&self) -> &[MemoryChunk] {
        &self.chunks
    }
}

fn validate_scope(
    scope: MemoryScope,
    project_id: Option<ProjectId>,
    agent_id: Option<AgentId>,
) -> Result<(), DomainError> {
    match scope {
        MemoryScope::Lab if project_id.is_none() && agent_id.is_none() => Ok(()),
        MemoryScope::Project if project_id.is_some() && agent_id.is_none() => Ok(()),
        MemoryScope::Agent if agent_id.is_some() && project_id.is_none() => Ok(()),
        _ => Err(DomainError::validation(
            "document.scope",
            "scope id does not match the document scope",
        )),
    }
}

/// True when the text contains a known secret shape.
#[must_use]
pub fn contains_secret(text: &str) -> bool {
    const PREFIXES: &[&str] = &["sk-", "ghp_", "github_pat_", "AKIA", "xoxb-"];
    text.split_whitespace().any(|token| {
        PREFIXES.iter().any(|prefix| token.starts_with(prefix))
            || token.contains("AILAB_MASTER_KEY=")
            || token.contains("password=")
            || text.contains("-----BEGIN")
    })
}

/// Deterministic word-based chunking: 800 words, 120 overlap.
#[must_use]
pub fn chunk_text(content: &str) -> Vec<String> {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    loop {
        let end = (start + CHUNK_WORDS).min(words.len());
        chunks.push(words[start..end].join(" "));
        if end == words.len() {
            break;
        }
        start = end - CHUNK_OVERLAP_WORDS;
    }
    chunks
}

/// Cosine similarity in `[-1, 1]`, or `None` for a dimension mismatch or zero norm.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn cosine_similarity(left: &[f32], right: &[f32]) -> Option<f64> {
    if left.len() != right.len() || left.is_empty() {
        return None;
    }
    let mut dot = 0.0_f64;
    let mut left_norm = 0.0_f64;
    let mut right_norm = 0.0_f64;
    for (a, b) in left.iter().zip(right) {
        let a = f64::from(*a);
        let b = f64::from(*b);
        dot = a.mul_add(b, dot);
        left_norm = a.mul_add(a, left_norm);
        right_norm = b.mul_add(b, right_norm);
    }
    if left_norm == 0.0 || right_norm == 0.0 {
        return None;
    }
    Some((dot / (left_norm.sqrt() * right_norm.sqrt())).clamp(-1.0, 1.0))
}

/// A retrieval query.
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    /// Lab scope.
    pub lab_id: LabId,
    /// Requested scope.
    pub scope: MemoryScope,
    /// Project id for project scope.
    pub project_id: Option<ProjectId>,
    /// Agent id for agent scope.
    pub agent_id: Option<AgentId>,
    /// Query text for the lexical component.
    pub query_text: String,
    /// Query embedding.
    pub query_embedding: Vec<f32>,
    /// Fixed clock.
    pub now: DateTime<Utc>,
    /// Number of hits.
    pub top_k: usize,
}

/// A retrieval hit.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    /// Chunk id.
    pub chunk_id: ChunkId,
    /// Document id.
    pub document_id: DocumentId,
    /// Hybrid score.
    pub score: f64,
    /// Cosine similarity.
    pub cosine: f64,
}

#[derive(Debug, Clone)]
struct Candidate {
    hit: SearchHit,
    embedding: Vec<f32>,
}

/// Hybrid retrieval with scope filtering, deduplication and citations.
#[allow(clippy::cast_precision_loss)]
pub fn hybrid_search(
    documents: &[MemoryDocument],
    query: &MemoryQuery,
) -> Result<Vec<SearchHit>, DomainError> {
    if query.query_embedding.len() != EMBEDDING_DIM {
        return Err(DomainError::validation(
            "query.embedding",
            "must have 384 dimensions",
        ));
    }
    if query.top_k == 0 {
        return Err(DomainError::validation("query.top_k", "must be positive"));
    }
    let terms: Vec<String> = query
        .query_text
        .to_lowercase()
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();
    let mut candidates = Vec::new();
    for document in documents {
        if document.lab_id != query.lab_id || document.scope != query.scope {
            continue;
        }
        if query.scope == MemoryScope::Project && document.project_id != query.project_id {
            continue;
        }
        if query.scope == MemoryScope::Agent && document.agent_id != query.agent_id {
            continue;
        }
        let age_days = (query.now - document.created_at).num_seconds() as f64 / 86_400.0;
        let recency = (-age_days.max(0.0) / 30.0).exp();
        for chunk in &document.chunks {
            let Some(cosine) = cosine_similarity(&chunk.embedding, &query.query_embedding) else {
                continue;
            };
            let lexical = lexical_rank(&terms, chunk.text());
            let weighted = 0.7_f64.mul_add(cosine.max(0.0), 0.3 * lexical);
            let score = weighted * recency * query.scope.boost();
            candidates.push(Candidate {
                hit: SearchHit {
                    chunk_id: chunk.id,
                    document_id: document.id,
                    score,
                    cosine,
                },
                embedding: chunk.embedding.clone(),
            });
        }
    }
    candidates.sort_by(|left, right| right.hit.score.total_cmp(&left.hit.score));
    let mut hits = Vec::new();
    for candidate in candidates {
        let near_duplicate = hits.iter().any(|kept: &Candidate| {
            cosine_similarity(&candidate.embedding, &kept.embedding)
                .is_some_and(|similarity| similarity >= DEDUPE_COSINE)
        });
        if near_duplicate {
            continue;
        }
        hits.push(candidate);
        if hits.len() == query.top_k {
            break;
        }
    }
    Ok(hits.into_iter().map(|candidate| candidate.hit).collect())
}

#[allow(clippy::cast_precision_loss)]
fn lexical_rank(terms: &[String], text: &str) -> f64 {
    if terms.is_empty() {
        return 0.0;
    }
    let lower = text.to_lowercase();
    let matches = terms
        .iter()
        .filter(|term| lower.contains(term.as_str()))
        .count();
    matches as f64 / terms.len() as f64
}

/// Episodic memory entry.
#[derive(Debug, Clone)]
pub struct Episode {
    /// Identity.
    pub id: uuid::Uuid,
    /// Lab scope.
    pub lab_id: LabId,
    /// Summary.
    pub summary: String,
    /// Pinned episodes are never compacted or deleted.
    pub pinned: bool,
    /// Creation time.
    pub created_at: DateTime<Utc>,
}

/// Compaction and deletion plan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompactionPlan {
    /// Episodes older than 30 days to compact.
    pub compact: Vec<uuid::Uuid>,
    /// Episodes older than 180 days to delete after compaction.
    pub delete_raw: Vec<uuid::Uuid>,
}

/// Plan compaction and raw retention.
#[must_use]
pub fn plan_compaction(episodes: &[Episode], now: DateTime<Utc>) -> CompactionPlan {
    let mut plan = CompactionPlan::default();
    for episode in episodes {
        if episode.pinned {
            continue;
        }
        let age_days = (now - episode.created_at).num_days();
        if age_days > 30 {
            plan.compact.push(episode.id);
        }
        if age_days > 180 {
            plan.delete_raw.push(episode.id);
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn emb(first: f32, second: f32) -> Vec<f32> {
        let mut vector = vec![0.0; EMBEDDING_DIM];
        vector[0] = first;
        vector[1] = second;
        vector
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 31, 0, 0, 0)
            .single()
            .expect("timestamp")
    }

    #[allow(clippy::needless_pass_by_value)]
    fn document(
        lab: LabId,
        scope: MemoryScope,
        project: Option<ProjectId>,
        agent: Option<AgentId>,
        text: &str,
        embedding: Vec<f32>,
        age_days: i64,
    ) -> MemoryDocument {
        MemoryDocument::new(
            DocumentId::generate(),
            lab,
            scope,
            project,
            agent,
            MemoryKind::Note,
            "Fixture",
            text,
            &|_| embedding.clone(),
            now() - Duration::days(age_days),
        )
        .expect("valid document")
    }

    #[test]
    fn scope_consistency_is_enforced() {
        let lab = LabId::generate();
        assert!(
            MemoryDocument::new(
                DocumentId::generate(),
                lab,
                MemoryScope::Lab,
                Some(ProjectId::generate()),
                None,
                MemoryKind::Note,
                "Bad",
                "text",
                &|_| emb(1.0, 0.0),
                now()
            )
            .is_err()
        );
    }

    #[test]
    fn secret_documents_are_quarantined() {
        let error = MemoryDocument::new(
            DocumentId::generate(),
            LabId::generate(),
            MemoryScope::Lab,
            None,
            None,
            MemoryKind::Note,
            "Secret",
            "token ghp_abcdef",
            &|_| emb(1.0, 0.0),
            now(),
        )
        .expect_err("quarantined");
        assert_eq!(error.code(), "MEMORY_QUARANTINED");
    }

    #[test]
    fn chunking_is_deterministic() {
        let content = (0..1000)
            .map(|index| format!("w{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let first = chunk_text(&content);
        let second = chunk_text(&content);
        assert_eq!(first, second);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].split_whitespace().count(), CHUNK_WORDS);
        assert_eq!(
            first[1].split_whitespace().count(),
            1000 - (CHUNK_WORDS - CHUNK_OVERLAP_WORDS)
        );
    }

    #[test]
    fn retrieval_is_scope_filtered() {
        let lab = LabId::generate();
        let project = ProjectId::generate();
        let documents = vec![
            document(
                lab,
                MemoryScope::Lab,
                None,
                None,
                "lab kanban",
                emb(1.0, 0.0),
                0,
            ),
            document(
                lab,
                MemoryScope::Project,
                Some(project),
                None,
                "project kanban",
                emb(1.0, 0.0),
                0,
            ),
            document(
                lab,
                MemoryScope::Agent,
                None,
                Some(AgentId::generate()),
                "agent kanban",
                emb(1.0, 0.0),
                0,
            ),
        ];
        let query = MemoryQuery {
            lab_id: lab,
            scope: MemoryScope::Project,
            project_id: Some(project),
            agent_id: None,
            query_text: "kanban".to_owned(),
            query_embedding: emb(1.0, 0.0),
            now: now(),
            top_k: 8,
        };
        let hits = hybrid_search(&documents, &query).expect("search");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document_id, documents[1].id());
    }

    #[test]
    fn hybrid_score_matches_the_formula() {
        let lab = LabId::generate();
        let project = ProjectId::generate();
        let documents = vec![document(
            lab,
            MemoryScope::Project,
            Some(project),
            None,
            "kanban card",
            emb(1.0, 0.0),
            0,
        )];
        let query = MemoryQuery {
            lab_id: lab,
            scope: MemoryScope::Project,
            project_id: Some(project),
            agent_id: None,
            query_text: "kanban card".to_owned(),
            query_embedding: emb(1.0, 0.0),
            now: now(),
            top_k: 8,
        };
        let hits = hybrid_search(&documents, &query).expect("search");
        let expected = 0.7_f64.mul_add(1.0, 0.3 * 1.0) * 1.0 * MemoryScope::Project.boost();
        assert!((hits[0].score - expected).abs() < 1e-9);
    }

    #[test]
    fn near_duplicates_are_deduped() {
        let lab = LabId::generate();
        let documents = vec![
            document(
                lab,
                MemoryScope::Lab,
                None,
                None,
                "kanban",
                emb(1.0, 0.0),
                0,
            ),
            document(
                lab,
                MemoryScope::Lab,
                None,
                None,
                "kanban",
                emb(1.0, 0.0),
                0,
            ),
        ];
        let query = MemoryQuery {
            lab_id: lab,
            scope: MemoryScope::Lab,
            project_id: None,
            agent_id: None,
            query_text: "kanban".to_owned(),
            query_embedding: emb(1.0, 0.0),
            now: now(),
            top_k: 8,
        };
        assert_eq!(hybrid_search(&documents, &query).expect("search").len(), 1);
    }

    #[test]
    fn compaction_respects_pins_and_retention() {
        let lab = LabId::generate();
        let old = Episode {
            id: uuid::Uuid::now_v7(),
            lab_id: lab,
            summary: "old".to_owned(),
            pinned: false,
            created_at: now() - Duration::days(200),
        };
        let recent = Episode {
            id: uuid::Uuid::now_v7(),
            lab_id: lab,
            summary: "recent".to_owned(),
            pinned: false,
            created_at: now() - Duration::days(10),
        };
        let mut pinned = old.clone();
        pinned.id = uuid::Uuid::now_v7();
        pinned.pinned = true;
        let plan = plan_compaction(&[old.clone(), recent, pinned.clone()], now());
        assert_eq!(plan.compact, vec![old.id]);
        assert_eq!(plan.delete_raw, vec![old.id]);
        assert!(!plan.compact.contains(&pinned.id));
    }
}
