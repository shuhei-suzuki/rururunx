# U4 Codex finite contract correspondence

Basis: `macos-minimum-m0-catalog-0162.json`, subtree `bbdede32736d0d954683bef90861c902a09aa46d33cf492f9f1cc3766005b38e`.
The wire field declarations in `execution/codex_wire/{types.rs,../codex_wire.rs}` were authored independently.
The schema gate compares declarations with static basis evidence; it never generates or runs a wire validator from CLI output.
The reduced pair contains only the 12 source roots and their local closures (eight ClientRequest methods), derived solely from that catalog.
`tests/fixtures/codex-contract/manifest.json` fixes provenance and all 20 scalar reach pairs.

The field-type notation is documented in `codex_wire/types.rs`. `Id` is a bounded string; `RpcId` additionally permits int64.
Every notification first uses the explicit 84-method ownership table; 21 then validate full payload. The remaining 63 keep bounded JSON only.
All 19 item kinds validate id/type; only agentMessage, commandExecution and fileChange validate content. New names exist only in a passed gate's memory contract.
The Turn wrapper exposes its original Value only for the existing whole-Turn integrity digest. The four existing whole-receipt checks remain unchanged.
ConfigRead uses only the typed judgement paths, with Opaque/tolerated fields inaccessible through `ConfigJudgement`.
The shared restrictive predicate is consumed by the legacy `DecisionPolicy` prefix; M0 never constructs that policy or reads its permission profile.
Inventory core is `data`, `nextCursor`, server `name`, `tools` map shape and `toolsError`; tool values are never inspected.

| Source | Definition | rrx type / field domain | Decoder | Consumer |
|---|---|---|---|---|
| ClientNotification | root | sent: initialized (no params) | CodexWireContract::sent | Handshake / Lines / collector / approval (according to method) |
| ClientRequest_used | root | sent: eight params records | CodexWireContract::sent | Handshake / Lines / collector / approval (according to method) |
| ClientRequest_used | AbsolutePathBuf | s (absolute-path consumer refinement) | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | AdditionalContextEntry | AdditionalContextEntry | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | AdditionalContextKind | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ApprovalsReviewer | ApprovalsReviewer | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | AskForApproval | AskForApproval | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ByteRange | ByteRange | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | CapabilityRootLocation | CapabilityRootLocation | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ClientInfo | ClientInfo | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | CollaborationMode | CollaborationMode | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ConfigReadParams | ConfigReadParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | CyberAccessProgram | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | DynamicToolNamespaceTool | DynamicToolNamespaceTool | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | DynamicToolSpec | DynamicToolSpec | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | EnvironmentStatusParams | EnvironmentStatusParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | FunctionCallOutputBody | FunctionOutput | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | FunctionCallOutputContentItem | FunctionContent | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | GetAccountParams | GetAccountParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | GetAccountRateLimitsParams | GetAccountRateLimitsParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ImageDetail | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | InitializeCapabilities | InitializeCapabilities | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | InitializeParams | InitializeParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | LegacyAppPathString | s | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ListMcpServerStatusParams | ListMcpServerStatusParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | McpServerStatusDetail | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ModeKind | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | MultiAgentMode | MultiAgentMode | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | NetworkAccess | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | Personality | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ReasoningEffort | Nonempty | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ReasoningSummary | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | RequestId | RpcId | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | SandboxMode | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | SandboxPolicy | SandboxPolicy | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | SelectedCapabilityRoot | SelectedCapabilityRoot | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | Settings | Settings | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | TextElement | TextElement | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ThreadHistoryMode | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ThreadSource | s | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ThreadStartParams | ThreadStartParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | ThreadStartSource | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | TurnEnvironmentParams | TurnEnvironmentParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | TurnStartParams | TurnStartParams | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | TurnToolOutput | TurnToolOutput | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ClientRequest_used | UserInput | UserInput | codex_wire::{check,types::check,variant} | Lines::call (sent) |
| ConfigReadResponse | root | ConfigReadResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| ConfigReadResponse | Config | Config | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ConfigReadResponse | WebSearchMode | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| EnvironmentStatusResponse | root | EnvironmentStatusResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| EnvironmentStatusResponse | EnvironmentStatusKind | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | root | GetAccountRateLimitsResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| GetAccountRateLimitsResponse | CreditsSnapshot | CreditsSnapshot | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | PlanType | PlanType | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | RateLimitReachedType | RateLimitReachedType | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | RateLimitResetCredit | RateLimitResetCredit | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | RateLimitResetCreditStatus | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | RateLimitResetCreditsSummary | RateLimitResetCreditsSummary | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | RateLimitResetType | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | RateLimitSnapshot | RateSnapshot | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | RateLimitWindow | RateLimitWindow | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountRateLimitsResponse | SpendControlLimitSnapshot | SpendControlLimitSnapshot | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountResponse | root | GetAccountResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| GetAccountResponse | Account | Account | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountResponse | AccountRoutingOverride | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountResponse | PlanType | PlanType | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| GetAccountResponse | WorkspaceRouting | WorkspaceRouting | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| InitializeResponse | root | InitializeResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| InitializeResponse | AbsolutePathBuf | s (absolute-path consumer refinement) | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ListMcpServerStatusResponse | root | ListMcpServerStatusResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| ListMcpServerStatusResponse | McpServerStatus | McpServerStatus | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ServerNotification | root | Frame + Ownership + strict notification params | CodexWireContract::notification | Handshake / Lines / collector / approval (according to method) |
| ServerNotification | AbsolutePathBuf | s (absolute-path consumer refinement) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AccountLoginCompletedNotification | Validated notification params [account/login/completed] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AccountRateLimitsUpdatedNotification | Validated notification params [account/rateLimits/updated] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AccountUpdatedNotification | Validated notification params [account/updated] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ActivePermissionProfile | ActivePermissionProfile | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AgentMessageDelivery | async | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AgentMessageDeltaNotification | Validated notification params [item/agentMessage/delta] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ApprovalsReviewer | ApprovalsReviewer | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AskForApproval | AskForApproval | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AsyncUserInputQuestion | AsyncUserInputQuestion | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | AuthMode | AuthMode | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | CodexErrorInfo | CodexErrorInfo | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | CollaborationMode | CollaborationMode | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | CommandAction | CommandAction | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | CommandExecutionSource | agent|userShell|unifiedExecStartup|unifiedExecInteraction | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | CommandExecutionStatus | inProgress|completed|failed|declined | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | CreditsSnapshot | CreditsSnapshot | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | DesktopOnboardingEntrypoint | DesktopOnboardingEntrypoint | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ErrorNotification | Validated notification params [error] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | FileChangePatchUpdatedNotification | Validated notification params [item/fileChange/patchUpdated] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | FileUpdateChange | FileUpdateChange | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ItemCompletedNotification | Validated notification params [item/completed] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ItemStartedNotification | Validated notification params [item/started] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | LegacyAppPathString | s | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | McpServerStartupFailureReason | McpServerStartupFailureReason | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | McpServerStartupState | starting|ready|failed|cancelled | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | McpServerStatusUpdatedNotification | Validated notification params [mcpServer/startupStatus/updated] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | MemoryCitation | MemoryCitation | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | MemoryCitationEntry | MemoryCitationEntry | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | MessagePhase | commentary|partial_answer|final_answer | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ModeKind | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | MultiAgentMode | MultiAgentMode | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | NetworkAccess | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | NonSteerableTurnKind | review|compact | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | PatchApplyStatus | inProgress|completed|failed|declined | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | PatchChangeKind | PatchChangeKind | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | Personality | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | PlanType | PlanType | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | RateLimitReachedType | RateLimitReachedType | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | RateLimitSnapshot | RateSnapshot | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | RateLimitWindow | RateLimitWindow | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ReasoningEffort | Nonempty | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ReasoningSummary | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | RequestId | RpcId | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | SandboxPolicy | SandboxPolicy | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ServerRequestResolvedNotification | Validated notification params [serverRequest/resolved] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | Settings | Settings | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | SpendControlLimitSnapshot | SpendControlLimitSnapshot | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | TerminalInteractionNotification | Validated notification params [item/commandExecution/terminalInteraction] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | Thread | Thread | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadActiveFlag | waitingOnApproval|waitingOnUserInput | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadArchivedNotification | Validated notification params [thread/archived] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadClosedNotification | Validated notification params [thread/closed] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadDeletedNotification | Validated notification params [thread/deleted] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadItem | ThreadItem | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadRevertedNotification | Validated notification params [thread/reverted] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadSettings | ThreadSettings | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadSettingsUpdatedNotification | Validated notification params [thread/settings/updated] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadStartedNotification | Validated notification params [thread/started] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadStatus | ThreadStatus | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadStatusChangedNotification | Validated notification params [thread/status/changed] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | ThreadUnarchivedNotification | Validated notification params [thread/unarchived] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | Turn | Turn | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | TurnCompletedNotification | Validated notification params [turn/completed] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | TurnError | TurnError | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | TurnItemsView | notLoaded|summary|full | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | TurnStartedNotification | Validated notification params [turn/started] | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerNotification | TurnStatus | completed|interrupted|failed|inProgress | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | root | Frame + eleven request params | CodexWireContract::server_request | Handshake / Lines / collector / approval (according to method) |
| ServerRequest | AdditionalFileSystemPermissions | AdditionalFileSystemPermissions | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | AdditionalNetworkPermissions | AdditionalNetworkPermissions | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | AdditionalPermissionProfile | AdditionalPermissionProfile | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ApplyPatchApprovalParams | ApplyPatchApprovalParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | AttestationGenerateParams | AttestationGenerateParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ChatgptAuthTokensRefreshParams | ChatgptAuthTokensRefreshParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ChatgptAuthTokensRefreshReason | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | CommandAction | CommandAction | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | CommandExecutionApprovalDecision | CommandExecutionApprovalDecision | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | CommandExecutionApprovalKind | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | CommandExecutionRequestApprovalParams | CommandExecutionRequestApprovalParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | CurrentTimeReadParams | CurrentTimeReadParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | DynamicToolCallParams | DynamicToolCallParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ExecCommandApprovalParams | ExecCommandApprovalParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | FileChange | FileChange | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | FileChangeRequestApprovalParams | FileChangeRequestApprovalParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | FileSystemAccessMode | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | FileSystemPath | FileSystemPath | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | FileSystemSandboxEntry | FileSystemSandboxEntry | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | FileSystemSpecialPath | SpecialPath | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | LegacyAppPathString | s | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationArrayType | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationBooleanSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationBooleanType | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationConstOption | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationLegacyTitledEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationMultiSelectEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationNumberSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationNumberType | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationObjectType | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationPrimitiveSchema | McpPrimitive | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationSingleSelectEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationStringFormat | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationStringSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationStringType | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationTitledEnumItems | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationTitledMultiSelectEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationTitledSingleSelectEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationUntitledEnumItems | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationUntitledMultiSelectEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpElicitationUntitledSingleSelectEnumSchema | McpPrimitive / McpElicitationSchema / McpConstOption / McpTitledItems / McpUntitledItems (closed records) | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | McpServerElicitationRequestParams | McpServerElicitationRequestParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | NetworkApprovalContext | NetworkApprovalContext | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | NetworkApprovalProtocol | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | NetworkPolicyAmendment | NetworkPolicyAmendment | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | NetworkPolicyRuleAction | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ParsedCommand | ParsedCommand | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | PermissionsRequestApprovalParams | PermissionsRequestApprovalParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | RequestId | RpcId | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | RequestPermissionProfile | RequestPermissionProfile | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ThreadId | Id | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ToolRequestUserInputOption | ToolRequestUserInputOption | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ToolRequestUserInputParams | ToolRequestUserInputParams | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ServerRequest | ToolRequestUserInputQuestion | ToolRequestUserInputQuestion | codex_wire::{check,types::check,variant} | managed::frame → ownership → collector / approval |
| ThreadStartResponse | root | ThreadStartResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| ThreadStartResponse | AbsolutePathBuf | s (absolute-path consumer refinement) | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | ActivePermissionProfile | ActivePermissionProfile | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | AgentMessageDelivery | async | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | ApprovalsReviewer | ApprovalsReviewer | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | AskForApproval | AskForApproval | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | AsyncUserInputQuestion | AsyncUserInputQuestion | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | CodexErrorInfo | CodexErrorInfo | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | CommandAction | CommandAction | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | CommandExecutionSource | agent|userShell|unifiedExecStartup|unifiedExecInteraction | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | CommandExecutionStatus | inProgress|completed|failed|declined | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | FileUpdateChange | FileUpdateChange | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | LegacyAppPathString | s | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | MemoryCitation | MemoryCitation | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | MemoryCitationEntry | MemoryCitationEntry | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | MessagePhase | commentary|partial_answer|final_answer | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | MultiAgentMode | MultiAgentMode | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | NetworkAccess | inline finite domain in field/variant declaration | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | NonSteerableTurnKind | review|compact | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | PatchApplyStatus | inProgress|completed|failed|declined | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | PatchChangeKind | PatchChangeKind | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | ReasoningEffort | Nonempty | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | SandboxPolicy | SandboxPolicy | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | Thread | Thread | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | ThreadActiveFlag | waitingOnApproval|waitingOnUserInput | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | ThreadItem | ThreadItem | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | ThreadStatus | ThreadStatus | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | Turn | Turn | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | TurnError | TurnError | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | TurnItemsView | notLoaded|summary|full | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| ThreadStartResponse | TurnStatus | completed|interrupted|failed|inProgress | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | root | TurnStartResponse | CodexWireContract::response | Handshake / Lines / collector / approval (according to method) |
| TurnStartResponse | AgentMessageDelivery | async | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | AsyncUserInputQuestion | AsyncUserInputQuestion | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | CodexErrorInfo | CodexErrorInfo | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | CommandAction | CommandAction | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | CommandExecutionSource | agent|userShell|unifiedExecStartup|unifiedExecInteraction | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | CommandExecutionStatus | inProgress|completed|failed|declined | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | FileUpdateChange | FileUpdateChange | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | LegacyAppPathString | s | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | MemoryCitation | MemoryCitation | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | MemoryCitationEntry | MemoryCitationEntry | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | MessagePhase | commentary|partial_answer|final_answer | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | NonSteerableTurnKind | review|compact | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | PatchApplyStatus | inProgress|completed|failed|declined | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | PatchChangeKind | PatchChangeKind | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | ThreadItem | ThreadItem | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | Turn | Turn | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | TurnError | TurnError | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | TurnItemsView | notLoaded|summary|full | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
| TurnStartResponse | TurnStatus | completed|interrupted|failed|inProgress | codex_wire::{check,types::check,variant} | Handshake / Lines::call |
