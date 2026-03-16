use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Evaluator type for a criterion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluatorType {
    Automated,
    HumanJudgment,
    AgentEvaluated,
}

impl EvaluatorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvaluatorType::Automated => "automated",
            EvaluatorType::HumanJudgment => "human_judgment",
            EvaluatorType::AgentEvaluated => "agent_evaluated",
        }
    }
}

impl FromStr for EvaluatorType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "automated" => Ok(EvaluatorType::Automated),
            "human_judgment" => Ok(EvaluatorType::HumanJudgment),
            "agent_evaluated" => Ok(EvaluatorType::AgentEvaluated),
            _ => Err(format!("invalid evaluator type: {s}")),
        }
    }
}

impl Serialize for EvaluatorType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EvaluatorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EvaluatorType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for EvaluatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Visibility of a criterion within a contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Visible => "visible",
            Visibility::Hidden => "hidden",
        }
    }
}

impl FromStr for Visibility {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "visible" => Ok(Visibility::Visible),
            "hidden" => Ok(Visibility::Hidden),
            _ => Err(format!("invalid visibility: {s}")),
        }
    }
}

impl Serialize for Visibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Visibility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Visibility::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Role within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionRole {
    Operator,
    Worker,
    Auditor,
}

impl SessionRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionRole::Operator => "operator",
            SessionRole::Worker => "worker",
            SessionRole::Auditor => "auditor",
        }
    }
}

impl FromStr for SessionRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "operator" => Ok(SessionRole::Operator),
            "worker" => Ok(SessionRole::Worker),
            "auditor" => Ok(SessionRole::Auditor),
            _ => Err(format!("invalid session role: {s}")),
        }
    }
}

impl fmt::Display for SessionRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Session state machine states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Discovering,
    Composing,
    Ready,
    Executing,
    AwaitingAdjudication,
    Adjudicated,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Discovering => "discovering",
            SessionStatus::Composing => "composing",
            SessionStatus::Ready => "ready",
            SessionStatus::Executing => "executing",
            SessionStatus::AwaitingAdjudication => "awaiting_adjudication",
            SessionStatus::Adjudicated => "adjudicated",
        }
    }

    /// Returns the valid next states from this state.
    pub fn valid_transitions(&self) -> &[SessionStatus] {
        match self {
            SessionStatus::Discovering => &[SessionStatus::Composing],
            SessionStatus::Composing => &[SessionStatus::Ready],
            SessionStatus::Ready => &[SessionStatus::Executing],
            SessionStatus::Executing => &[SessionStatus::AwaitingAdjudication],
            SessionStatus::AwaitingAdjudication => &[SessionStatus::Adjudicated],
            SessionStatus::Adjudicated => &[],
        }
    }

    /// Check if transitioning to `next` is valid.
    pub fn can_transition_to(&self, next: &SessionStatus) -> bool {
        self.valid_transitions().contains(next)
    }
}

impl FromStr for SessionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "discovering" => Ok(SessionStatus::Discovering),
            "composing" => Ok(SessionStatus::Composing),
            "ready" => Ok(SessionStatus::Ready),
            "executing" => Ok(SessionStatus::Executing),
            "awaiting_adjudication" => Ok(SessionStatus::AwaitingAdjudication),
            "adjudicated" => Ok(SessionStatus::Adjudicated),
            _ => Err(format!("invalid session status: {s}")),
        }
    }
}

impl Serialize for SessionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SessionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SessionStatus::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Decision type for adjudication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionType {
    Accept,
    Reject,
    AcceptResidual,
    InsufficientEvidence,
    WaiveCriterion,
    RequireRework,
    Rescope,
}

impl DecisionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionType::Accept => "accept",
            DecisionType::Reject => "reject",
            DecisionType::AcceptResidual => "accept_residual",
            DecisionType::InsufficientEvidence => "insufficient_evidence",
            DecisionType::WaiveCriterion => "waive_criterion",
            DecisionType::RequireRework => "require_rework",
            DecisionType::Rescope => "rescope",
        }
    }
}

impl FromStr for DecisionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "accept" => Ok(DecisionType::Accept),
            "reject" => Ok(DecisionType::Reject),
            "accept_residual" => Ok(DecisionType::AcceptResidual),
            "insufficient_evidence" => Ok(DecisionType::InsufficientEvidence),
            "waive_criterion" => Ok(DecisionType::WaiveCriterion),
            "require_rework" => Ok(DecisionType::RequireRework),
            "rescope" => Ok(DecisionType::Rescope),
            _ => Err(format!("invalid decision type: {s}")),
        }
    }
}

impl Serialize for DecisionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DecisionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DecisionType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for DecisionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Tier of a criterion (how critical it is).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tier {
    Must,
    Should,
    Nice,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Must => "must",
            Tier::Should => "should",
            Tier::Nice => "nice",
        }
    }
}

impl FromStr for Tier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "must" => Ok(Tier::Must),
            "should" => Ok(Tier::Should),
            "nice" => Ok(Tier::Nice),
            _ => Err(format!("invalid tier: {s}")),
        }
    }
}

impl Serialize for Tier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Tier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Tier::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Independence level of evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Independence {
    ToolAuthored,
    WorkerNarration,
    HumanObserved,
}

impl Independence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Independence::ToolAuthored => "tool_authored",
            Independence::WorkerNarration => "worker_narration",
            Independence::HumanObserved => "human_observed",
        }
    }
}

impl FromStr for Independence {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tool_authored" => Ok(Independence::ToolAuthored),
            "worker_narration" => Ok(Independence::WorkerNarration),
            "human_observed" => Ok(Independence::HumanObserved),
            _ => Err(format!("invalid independence: {s}")),
        }
    }
}

impl Serialize for Independence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Independence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Independence::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for Independence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Export format for decision logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Markdown,
}

impl FromStr for ExportFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(ExportFormat::Json),
            "markdown" => Ok(ExportFormat::Markdown),
            _ => Err(format!("invalid export format: {s}")),
        }
    }
}

/// Discovery source type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoverySourceType {
    File,
    GitLog,
    CiConfig,
    Doc,
}

impl DiscoverySourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiscoverySourceType::File => "file",
            DiscoverySourceType::GitLog => "git_log",
            DiscoverySourceType::CiConfig => "ci_config",
            DiscoverySourceType::Doc => "doc",
        }
    }
}

impl FromStr for DiscoverySourceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(DiscoverySourceType::File),
            "git_log" => Ok(DiscoverySourceType::GitLog),
            "ci_config" => Ok(DiscoverySourceType::CiConfig),
            "doc" => Ok(DiscoverySourceType::Doc),
            _ => Err(format!("invalid discovery source type: {s}")),
        }
    }
}

impl Serialize for DiscoverySourceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DiscoverySourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DiscoverySourceType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for DiscoverySourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Domain Structs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub claim: String,
    pub evaluator_type: EvaluatorType,
    pub check_spec: String,
    pub parameter_schema: Option<String>,
    pub created_at: String,
    pub deprecated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: String,
    pub session_id: String,
    pub goal: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCriterion {
    pub contract_id: String,
    pub criterion_id: String,
    pub visibility: Visibility,
    pub base_tier: Tier,
    pub base_independence: Independence,
    pub parameters: Option<String>,
    pub residual_claim: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub contract_id: Option<String>,
    pub worktree_path: String,
    pub status: SessionStatus,
    pub worker_token: Option<String>,
    pub operator_token: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryContext {
    pub id: String,
    pub session_id: String,
    pub source_type: DiscoverySourceType,
    pub source_path: String,
    pub content_hash: String,
    pub summary: String,
    pub gathered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub session_id: String,
    pub criterion_id: String,
    pub command_run: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub effective_tier: Tier,
    pub effective_independence: Independence,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub contract_id: String,
    pub decision_type: DecisionType,
    pub actor: String,
    pub basis: String,
    pub evidence_refs: String,
    pub resolves: String,
    pub outcome: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: i64,
    pub contract_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: String,
    pub payload: String,
    pub created_at: String,
}

/// Serializable TOML representation of a criterion for export/import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionToml {
    pub namespace: String,
    pub name: String,
    pub claim: String,
    pub evaluator_type: EvaluatorType,
    pub check_spec: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_schema: Option<String>,
}

impl From<&Criterion> for CriterionToml {
    fn from(c: &Criterion) -> Self {
        CriterionToml {
            namespace: c.namespace.clone(),
            name: c.name.clone(),
            claim: c.claim.clone(),
            evaluator_type: c.evaluator_type.clone(),
            check_spec: c.check_spec.clone(),
            parameter_schema: c.parameter_schema.clone(),
        }
    }
}

/// Input for non-interactive contract composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFromInput {
    /// The goal statement for the contract.
    pub goal: String,
    /// The criteria to bind to the contract.
    pub criteria: Vec<ComposeFromCriterion>,
}

/// A criterion definition within a compose-from input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFromCriterion {
    /// Criterion namespace.
    pub namespace: String,
    /// Criterion name.
    pub name: String,
    /// What the criterion claims.
    pub claim: String,
    /// How the criterion is evaluated.
    pub evaluator_type: EvaluatorType,
    /// Command or spec to verify.
    pub check_spec: String,
    /// Visibility within the contract.
    pub visibility: Visibility,
    /// Tier (criticality).
    pub tier: Tier,
    /// Optional JSON schema for parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_schema: Option<String>,
    /// Optional residual claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual_claim: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_status_transitions() {
        assert!(SessionStatus::Discovering.can_transition_to(&SessionStatus::Composing));
        assert!(!SessionStatus::Discovering.can_transition_to(&SessionStatus::Ready));
        assert!(SessionStatus::Composing.can_transition_to(&SessionStatus::Ready));
        assert!(SessionStatus::Ready.can_transition_to(&SessionStatus::Executing));
        assert!(SessionStatus::Executing.can_transition_to(&SessionStatus::AwaitingAdjudication));
        assert!(
            SessionStatus::AwaitingAdjudication.can_transition_to(&SessionStatus::Adjudicated)
        );
        assert!(!SessionStatus::Adjudicated.can_transition_to(&SessionStatus::Discovering));
    }

    #[test]
    fn evaluator_type_roundtrip() {
        for et in [
            EvaluatorType::Automated,
            EvaluatorType::HumanJudgment,
            EvaluatorType::AgentEvaluated,
        ] {
            let s = et.as_str();
            let parsed: EvaluatorType = s.parse().unwrap();
            assert_eq!(et, parsed);
        }
    }

    #[test]
    fn visibility_roundtrip() {
        for v in [Visibility::Visible, Visibility::Hidden] {
            let s = v.as_str();
            let parsed: Visibility = s.parse().unwrap();
            assert_eq!(v, parsed);
        }
    }

    #[test]
    fn tier_roundtrip() {
        for t in [Tier::Must, Tier::Should, Tier::Nice] {
            let s = t.as_str();
            let parsed: Tier = s.parse().unwrap();
            assert_eq!(t, parsed);
        }
    }

    #[test]
    fn decision_type_roundtrip() {
        for dt in [
            DecisionType::Accept,
            DecisionType::Reject,
            DecisionType::AcceptResidual,
            DecisionType::InsufficientEvidence,
            DecisionType::WaiveCriterion,
            DecisionType::RequireRework,
            DecisionType::Rescope,
        ] {
            let s = dt.as_str();
            let parsed: DecisionType = s.parse().unwrap();
            assert_eq!(dt, parsed);
        }
    }

    #[test]
    fn criterion_toml_roundtrip() {
        let c = Criterion {
            id: "test".to_string(),
            namespace: "testing".to_string(),
            name: "unit-tests".to_string(),
            claim: "All unit tests pass".to_string(),
            evaluator_type: EvaluatorType::Automated,
            check_spec: "cargo test".to_string(),
            parameter_schema: None,
            created_at: "2024-01-01".to_string(),
            deprecated_at: None,
        };
        let toml_repr = CriterionToml::from(&c);
        let serialized = toml::to_string(&toml_repr).unwrap();
        let deserialized: CriterionToml = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.namespace, "testing");
        assert_eq!(deserialized.name, "unit-tests");
        assert_eq!(deserialized.claim, "All unit tests pass");
    }
}
