//! Hand-authored field types. `?field` is optional, `~` nullable, `[]` array,
//! `{}` a typed map; none of these definitions is generated from JSON schema.
use super::*;

pub(super) fn check(ty: &str, v: &Value, c: &CodexWireContract) -> Result<()> {
    let fields = match ty {
        "InitializeParams" => "clientInfo:ClientInfo ?capabilities:InitializeCapabilities~",
        "ClientInfo" => "name:s version:s ?title:s~",
        "InitializeCapabilities" => {
            "?experimentalApi:b ?explicitGatewayOauth:b ?extensions:opaque{}~ ?mcpServerOpenaiFormElicitation:b ?optOutNotificationMethods:s[]~ ?requestAttestation:b"
        }
        "GetAccountParams" => "?refreshToken:b",
        "GetAccountRateLimitsParams" => "?excludeResetCreditDetails:b ?supportsLunaReserve:b",
        "EnvironmentStatusParams" => "environmentId:s",
        "ConfigReadParams" => "?cwd:s~ ?includeLayers:b",
        "ListMcpServerStatusParams" => {
            "?cursor:s~ ?detail:full|toolsAndAuthOnly~ ?limit:u32~ ?serverName:s~ ?threadId:Id~"
        }
        "ThreadStartParams" => {
            "?allowProviderModelFallback:b ?approvalPolicy:AskForApproval~ ?approvalsReviewer:ApprovalsReviewer~ ?baseInstructions:s~ ?config:opaque{}~ ?cwd:s~ ?daybreakEnabled:b~ ?developerInstructions:s~ ?dynamicTools:DynamicToolSpec[]~ ?environments:TurnEnvironmentParams[]~ ?ephemeral:b~ ?experimentalRawEvents:b ?historyMode:legacy|paginated~ ?mockExperimentalField:s~ ?model:s~ ?modelProvider:s~ ?multiAgentMode:MultiAgentMode~ ?permissions:s~ ?personality:none|friendly|pragmatic~ ?projectId:s~ ?runtimeWorkspaceRoots:s[]~ ?sandbox:read-only|workspace-write|danger-full-access~ ?selectedCapabilityRoots:SelectedCapabilityRoot[]~ ?serviceName:s~ ?serviceTier:s~ ?sessionStartSource:startup|clear~ ?threadSource:s~"
        }
        "TurnStartParams" => {
            "input:UserInput[] threadId:Id ?additionalContext:AdditionalContextEntry{}~ ?approvalPolicy:AskForApproval~ ?approvalsReviewer:ApprovalsReviewer~ ?clientUserMessageId:s~ ?collaborationMode:CollaborationMode~ ?cwd:s~ ?cyberAccessProgram:standard|daybreakBlue|daybreakRed~ ?disabledPluginIds:s[]~ ?effort:Nonempty~ ?environments:TurnEnvironmentParams[]~ ?model:s~ ?multiAgentMode:MultiAgentMode~ ?outputSchema:opaque ?parentTurnId:s~ ?permissions:s~ ?personality:none|friendly|pragmatic~ ?responsesapiClientMetadata:s{}~ ?rootTurnId:s~ ?runtimeWorkspaceRoots:s[]~ ?sandboxPolicy:SandboxPolicy~ ?serviceTier:s~ ?serviceTierForTurn:s~ ?summary:auto|concise|detailed|none~ ?toolOutput:TurnToolOutput~ ?turnTrigger:s~"
        }
        "AdditionalContextEntry" => "kind:untrusted|application value:s",
        "TurnToolOutput" => "name:s output:FunctionOutput ?namespace:s~",
        "TurnEnvironmentParams" => "cwd:s environmentId:s ?runtimeWorkspaceRoots:s[]~",
        "SelectedCapabilityRoot" => "id:s location:CapabilityRootLocation",
        "CapabilityRootLocation" => "type:=environment path:s environmentId:s",
        "CollaborationMode" => "mode:plan|default settings:Settings",
        "Settings" => "model:s ?developer_instructions:s~ ?reasoning_effort:Nonempty~",
        "InitializeResponse" => "codexHome:s platformFamily:s platformOs:s userAgent:s",
        "GetAccountResponse" => {
            "requiresOpenaiAuth:b ?account:Account~ ?workspaceRouting:WorkspaceRouting~"
        }
        "WorkspaceRouting" => {
            "accountRoutingOverride:NO_CONSTRAINT|us|us_cr backendOrigin:s chatgptAccountId:s"
        }
        "EnvironmentStatusResponse" => "status:ready|pending|disconnected|unknown ?error:s~",
        "GetAccountRateLimitsResponse" => {
            "rateLimits:RateSnapshot ?accountId:s~ ?ordinaryUsageAllowed:b~ ?rateLimitResetCredits:RateLimitResetCreditsSummary~ ?rateLimitUpsell:opaque ?rateLimitsByLimitId:RateSnapshot{}~"
        }
        "RateSnapshot" => {
            "?credits:CreditsSnapshot~ ?individualLimit:SpendControlLimitSnapshot~ ?limitId:s~ ?limitName:s~ ?normalModelSlug:s~ ?planType:PlanType~ ?primary:RateLimitWindow~ ?secondary:RateLimitWindow~ ?rateLimitReachedType:RateLimitReachedType~ ?spendControlReached:b~"
        }
        "CreditsSnapshot" => "hasCredits:b unlimited:b ?balance:s~",
        "SpendControlLimitSnapshot" => "limit:s remainingPercent:i32 resetsAt:i used:s",
        "RateLimitWindow" => "usedPercent:i32 ?resetsAt:i~ ?windowDurationMins:i~",
        "RateLimitResetCreditsSummary" => "availableCount:i ?credits:RateLimitResetCredit[]~",
        "RateLimitResetCredit" => {
            "grantedAt:i id:s resetType:codexRateLimits|unknown status:available|redeeming|redeemed|unknown ?description:s~ ?expiresAt:i~ ?title:s~"
        }
        "ThreadStartResponse" => {
            "approvalPolicy:AskForApproval approvalsReviewer:ApprovalsReviewer cwd:s model:s modelProvider:s sandbox:SandboxPolicy thread:Thread ?activePermissionProfile:ActivePermissionProfile~ ?disabledPluginIds:s[] ?instructionSources:s[] ?multiAgentMode:MultiAgentMode ?reasoningEffort:Nonempty~ ?runtimeWorkspaceRoots:s[] ?serviceTier:s~"
        }
        "ActivePermissionProfile" => "id:s ?extends:s~",
        "Thread" => "id:Id cwd:s modelProvider:s status:ThreadStatus turns:Turn[]",
        "ThreadSettings" => {
            "approvalPolicy:AskForApproval approvalsReviewer:ApprovalsReviewer collaborationMode:CollaborationMode cwd:s model:s modelProvider:s sandboxPolicy:SandboxPolicy ?activePermissionProfile:ActivePermissionProfile~ ?disabledPluginIds:s[] ?effort:Nonempty~ ?multiAgentMode:MultiAgentMode ?personality:none|friendly|pragmatic~ ?serviceTier:s~ ?summary:auto|concise|detailed|none~"
        }
        "TurnStartResponse" => "turn:Turn",
        "Turn" => {
            "id:Id items:ThreadItem[] status:completed|interrupted|failed|inProgress ?completedAt:i~ ?durationMs:i~ ?error:TurnError~ ?itemsView:notLoaded|summary|full ?rootTurnId:s~ ?startedAt:i~"
        }
        "TurnError" => {
            "message:s ?additionalDetails:s~ ?codexErrorInfo:CodexErrorInfo~ ?misalignment:opaque"
        }
        "AsyncUserInputQuestion" => return record(v, "title:s ?options:s[]~", true, c),
        "MemoryCitation" => "entries:MemoryCitationEntry[] threadIds:s[]",
        "MemoryCitationEntry" => "path:s lineStart:u32 lineEnd:u32 note:s",
        "FileUpdateChange" => "diff:s kind:PatchChangeKind path:s",
        "CommandExecutionRequestApprovalParams" => {
            "itemId:Id startedAtMs:i threadId:Id turnId:Id ?additionalPermissions:AdditionalPermissionProfile~ ?approvalId:s~ ?availableDecisions:CommandExecutionApprovalDecision[]~ ?command:s~ ?commandActions:CommandAction[]~ ?cwd:s~ ?environmentId:s~ ?kind:command|writeStdin ?networkApprovalContext:NetworkApprovalContext~ ?proposedExecpolicyAmendment:s[]~ ?proposedNetworkPolicyAmendments:NetworkPolicyAmendment[]~ ?reason:s~"
        }
        "FileChangeRequestApprovalParams" => {
            "itemId:Id startedAtMs:i threadId:Id turnId:Id ?grantRoot:s~ ?reason:s~"
        }
        "ToolRequestUserInputParams" => {
            "isBlocking:b itemId:Id questions:ToolRequestUserInputQuestion[] threadId:Id turnId:Id ?autoResolutionMs:u~"
        }
        "ToolRequestUserInputQuestion" => {
            "header:s id:s question:s ?isOther:b ?isSecret:b ?options:ToolRequestUserInputOption[]~"
        }
        "ToolRequestUserInputOption" => "description:s label:s",
        "PermissionsRequestApprovalParams" => {
            "cwd:s itemId:Id permissions:RequestPermissionProfile startedAtMs:i threadId:Id turnId:Id ?environmentId:s~ ?reason:s~"
        }
        "DynamicToolCallParams" => {
            "arguments:opaque callId:s threadId:Id tool:s turnId:Id ?namespace:s~"
        }
        "ChatgptAuthTokensRefreshParams" => "reason:=unauthorized ?previousAccountId:s~",
        "AttestationGenerateParams" => "",
        "CurrentTimeReadParams" => "threadId:Id",
        "ApplyPatchApprovalParams" => {
            "callId:s conversationId:Id fileChanges:FileChange{} ?grantRoot:s~ ?reason:s~"
        }
        "ExecCommandApprovalParams" => {
            "callId:s command:s[] conversationId:Id cwd:s parsedCmd:ParsedCommand[] ?approvalId:s~ ?reason:s~"
        }
        "AdditionalPermissionProfile" => {
            "?fileSystem:AdditionalFileSystemPermissions~ ?network:AdditionalNetworkPermissions~"
        }
        "RequestPermissionProfile" => {
            return record(
                v,
                "?fileSystem:AdditionalFileSystemPermissions~ ?network:AdditionalNetworkPermissions~",
                true,
                c,
            );
        }
        "AdditionalFileSystemPermissions" => {
            record(
                v,
                "?entries:FileSystemSandboxEntry[]~ ?globScanMaxDepth:u~ ?read:s[]~ ?write:s[]~",
                false,
                c,
            )?;
            ensure!(
                v.get("globScanMaxDepth")
                    .is_none_or(|n| n.is_null() || n.as_u64().is_some_and(|n| n >= 1)),
                "invalid glob depth"
            );
            return Ok(());
        }
        "AdditionalNetworkPermissions" => "?enabled:b~",
        "FileSystemSandboxEntry" => "access:read|write|deny path:FileSystemPath",
        "NetworkApprovalContext" => "host:s protocol:http|https|socks5Tcp|socks5Udp",
        "NetworkPolicyAmendment" => "action:allow|deny host:s",
        "ByteRange" => "start:u end:u",
        "TextElement" => "byteRange:ByteRange ?placeholder:s~",
        "DynamicToolNamespaceTool" => {
            "type:=function description:s inputSchema:opaque name:s ?deferLoading:b"
        }
        "ConfigReadResponse" => {
            record(
                v,
                "config:Config origins:opaque{} ?layers:opaque[]~",
                false,
                c,
            )?;
            ensure!(
                v.get("layers").is_none_or(Value::is_null),
                ExecutorCapabilityUnverified
            );
            return Ok(());
        }
        "Config" => {
            record(v, "?web_search:disabled|cached|indexed|live~", false, c)?;
            record(v, "?features:b{}~ ?mcp_servers:McpConfigEntry{}~", false, c)
                .map_err(|_| anyhow::Error::from(ExecutorCapabilityUnverified))?;
            return Ok(());
        }
        "McpConfigEntry" => "?enabled:b",
        "ListMcpServerStatusResponse" => "data:McpServerStatus[] ?nextCursor:s~",
        "McpServerStatus" => "name:s tools:opaque{} ?toolsError:s~",
        // Finite enums, including leaves that remain open strings in the basis.
        "PlanType" => {
            return super::check(
                "free|go|plus|pro|prolite|promax|team|self_serve_business_prolite|self_serve_business_usage_based|business|ent26|enterprise_cbp_automation|enterprise_cbp_usage_based|enterprise|edu|edu_plus|edu_pro|unknown",
                v,
                c,
            );
        }
        "ApprovalsReviewer" => return super::check("user|auto_review|guardian_subagent", v, c),
        "RateLimitReachedType" => {
            return super::check(
                "rate_limit_reached|workspace_owner_credits_depleted|workspace_member_credits_depleted|workspace_owner_usage_limit_reached|workspace_member_usage_limit_reached",
                v,
                c,
            );
        }
        "AuthMode" => {
            return super::check(
                "apikey|chatgpt|chatgptAuthTokens|headers|agentIdentity|personalAccessToken|bedrockApiKey|bedrockAccessKeys",
                v,
                c,
            );
        }
        "McpServerStartupFailureReason" => return super::check("=reauthenticationRequired", v, c),
        "DesktopOnboardingEntrypoint" => return super::check("=life_sciences", v, c),
        "CodexErrorInfo" => {
            error_kind(v)?;
            return Ok(());
        }
        _ => return variant(ty, v, c),
    };
    record(v, fields, false, c)
}

fn tag<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .context("Codex discriminator missing")
}
fn variant(ty: &str, v: &Value, c: &CodexWireContract) -> Result<()> {
    let fields = match ty {
        "ThreadItem" => {
            record(v, "id:Id type:s", false, c)?;
            let kind = tag(v, "type")?;
            ensure!(
                ITEM_TYPES.contains(&kind) || c.items.iter().any(|s| s == kind),
                "unknown Codex item type"
            );
            match kind {
                "agentMessage" => {
                    "text:s ?delivery:=async~ ?memoryCitation:MemoryCitation~ ?phase:commentary|partial_answer|final_answer~ ?questions:AsyncUserInputQuestion[]~"
                }
                "commandExecution" => {
                    "command:s commandActions:CommandAction[] cwd:s status:inProgress|completed|failed|declined ?aggregatedOutput:s~ ?durationMs:i~ ?exitCode:i32~ ?pluginId:s~ ?processId:s~ ?scriptPath:s~ ?source:agent|userShell|unifiedExecStartup|unifiedExecInteraction"
                }
                "fileChange" => {
                    "changes:FileUpdateChange[] status:inProgress|completed|failed|declined"
                }
                _ => "",
            }
        }
        "ThreadStatus" => match tag(v, "type")? {
            "notLoaded" | "idle" | "systemError" => "type:s",
            "active" => "type:s activeFlags:waitingOnApproval|waitingOnUserInput[]",
            _ => bail!("unknown thread status"),
        },
        "SandboxPolicy" => match tag(v, "type")? {
            "dangerFullAccess" => "type:s",
            "readOnly" => "type:s ?networkAccess:b",
            "externalSandbox" => "type:s ?networkAccess:restricted|enabled",
            "workspaceWrite" => {
                "type:s ?excludeSlashTmp:b ?excludeTmpdirEnvVar:b ?networkAccess:b ?writableRoots:s[]"
            }
            _ => bail!("unknown sandbox policy"),
        },
        "AskForApproval" => {
            if v.is_string() {
                return super::check("untrusted|on-request|never", v, c);
            }
            return record(v, "granular:GranularApproval", true, c);
        }
        "GranularApproval" => {
            "mcp_elicitations:b rules:b sandbox_approval:b ?request_permissions:b ?skill_approval:b"
        }
        "MultiAgentMode" => {
            if v.is_string() {
                return super::check("explicitRequestOnly|proactive", v, c);
            }
            return record(v, "custom:s", true, c);
        }
        "Account" => match tag(v, "type")? {
            "apiKey" => "type:s",
            "chatgpt" => "type:s email:s~ planType:PlanType",
            "amazonBedrock" => "type:s ?usesCodexManagedCredentials:b",
            _ => bail!("unknown Codex account type"),
        },
        "CommandAction" => match tag(v, "type")? {
            "read" => "type:s command:s name:s path:s",
            "listFiles" => "type:s command:s ?path:s~",
            "search" => "type:s command:s ?path:s~ ?query:s~",
            "unknown" => "type:s command:s",
            _ => bail!("unknown command action"),
        },
        "ParsedCommand" => match tag(v, "type")? {
            "read" => "type:s cmd:s name:s path:s",
            "list_files" => "type:s cmd:s ?path:s~",
            "search" => "type:s cmd:s ?path:s~ ?query:s~",
            "unknown" => "type:s cmd:s",
            _ => bail!("unknown parsed command"),
        },
        "PatchChangeKind" => match tag(v, "type")? {
            "add" | "delete" => "type:s",
            "update" => "type:s ?move_path:s~",
            _ => bail!("unknown patch change kind"),
        },
        "FileChange" => match tag(v, "type")? {
            "add" | "delete" => "type:s content:s",
            "update" => "type:s unified_diff:s ?move_path:s~",
            _ => bail!("unknown file change"),
        },
        "CommandExecutionApprovalDecision" => {
            if v.is_string() {
                return super::check("accept|acceptForSession|decline|cancel", v, c);
            }
            return key_tag(
                v,
                &[
                    ("acceptWithExecpolicyAmendment", "ExecpolicyAmendment"),
                    ("applyNetworkPolicyAmendment", "ApplyNetworkAmendment"),
                ],
                c,
            );
        }
        "ExecpolicyAmendment" => "execpolicy_amendment:s[]",
        "ApplyNetworkAmendment" => "network_policy_amendment:NetworkPolicyAmendment",
        "FileSystemPath" => match tag(v, "type")? {
            "path" => "type:s path:s",
            "glob_pattern" => "type:s pattern:s",
            "special" => "type:s value:SpecialPath",
            _ => bail!("unknown filesystem path"),
        },
        "SpecialPath" => match tag(v, "kind")? {
            "root" | "minimal" | "tmpdir" | "slash_tmp" => "kind:s",
            "project_roots" => "kind:s ?subpath:s~",
            "unknown" => "kind:s path:s ?subpath:s~",
            _ => bail!("unknown filesystem special path"),
        },
        "UserInput" => match tag(v, "type")? {
            "text" => "type:s text:s ?text_elements:TextElement[]",
            "image" => {
                image(v, "url", "fileId", c)?;
                "type:s ?detail:auto|low|high|original~"
            }
            "localImage" => "type:s path:s ?detail:auto|low|high|original~",
            "audio" => "type:s url:s",
            "localAudio" => "type:s path:s",
            "skill" | "mention" => "type:s name:s path:s",
            _ => bail!("unknown user input"),
        },
        "FunctionOutput" => {
            return if v.is_string() {
                Ok(())
            } else {
                super::check("FunctionContent[]", v, c)
            };
        }
        "FunctionContent" => match tag(v, "type")? {
            "input_text" => "type:s text:s",
            "input_image" => {
                image(v, "image_url", "file_id", c)?;
                "type:s ?detail:auto|low|high|original~"
            }
            "input_audio" => "type:s audio_url:s",
            "encrypted_content" => "type:s encrypted_content:s",
            _ => bail!("unknown function content"),
        },
        "DynamicToolSpec" => match tag(v, "type")? {
            "function" => "type:s description:s inputSchema:opaque name:s ?deferLoading:b",
            "namespace" => "type:s description:s name:s tools:DynamicToolNamespaceTool[]",
            _ => bail!("unknown dynamic tool type"),
        },
        "McpServerElicitationRequestParams" => {
            record(
                v,
                "serverName:s threadId:Id ?turnId:Id~ ?_meta:opaque",
                false,
                c,
            )?;
            match tag(v, "mode")? {
                "openai/userVerification" => "mode:s challenge:s description:s title:s",
                "form" => "mode:s message:s requestedSchema:McpElicitationSchema",
                "openai/form" | "openaiForm" => "mode:s message:s requestedSchema:opaque",
                "url" => "mode:s elicitationId:s message:s url:s",
                _ => bail!("unknown elicitation mode"),
            }
        }
        "McpElicitationSchema" => {
            return record(
                v,
                "type:=object properties:McpPrimitive{} ?$schema:s~ ?required:s[]~",
                true,
                c,
            );
        }
        "McpPrimitive" => return primitive(v, c),
        "McpConstOption" => return record(v, "const:s title:s", true, c),
        "McpUntitledItems" => return record(v, "type:=string enum:s[]", true, c),
        "McpTitledItems" => return record(v, "anyOf:McpConstOption[]", true, c),
        _ => bail!("internal finite Codex type missing: {ty}"),
    };
    record(v, fields, false, c)
}
fn image(v: &Value, url: &str, file: &str, c: &CodexWireContract) -> Result<()> {
    ensure!(
        v.get(url).is_some() || v.get(file).is_some(),
        "image source absent"
    );
    for k in [url, file] {
        check_optional(v.get(k), "s", c)?;
    }
    Ok(())
}
fn primitive(v: &Value, c: &CodexWireContract) -> Result<()> {
    let common = "?description:s~ ?title:s~";
    record(v, common, false, c)?;
    let tail = match tag(v, "type")? {
        "boolean" => "type:=boolean ?default:b~",
        "number" | "integer" => "type:number|integer ?default:n~ ?minimum:n~ ?maximum:n~",
        "string" if v.get("oneOf").is_some() => "type:=string oneOf:McpConstOption[] ?default:s~",
        "string" if v.get("enum").is_some() => "type:=string enum:s[] ?enumNames:s[]~ ?default:s~",
        "string" => {
            "type:=string ?default:s~ ?format:email|uri|date|date-time~ ?maxLength:u32~ ?minLength:u32~"
        }
        "array" => {
            let fields = if v["items"].get("anyOf").is_some() {
                "McpTitledItems"
            } else {
                "McpUntitledItems"
            };
            super::check(fields, &v["items"], c)?;
            "type:=array items:opaque ?default:s[]~ ?maxItems:u~ ?minItems:u~"
        }
        _ => bail!("unknown MCP primitive"),
    };
    record(v, &format!("{common} {tail}"), true, c)
}
fn key_tag(v: &Value, tags: &[(&str, &str)], c: &CodexWireContract) -> Result<()> {
    let o = v.as_object().context("key-tag object required")?;
    let present = tags
        .iter()
        .filter(|(k, _)| o.contains_key(*k))
        .collect::<Vec<_>>();
    ensure!(
        present.len() == 1 && o.len() == 1,
        "ambiguous or extended key-tag union"
    );
    super::check(present[0].1, &o[present[0].0], c)
}
pub(super) fn error_kind(v: &Value) -> Result<CodexErrorKind> {
    use CodexErrorKind::*;
    if let Some(s) = v.as_str() {
        return Ok(match s {
            "usageLimitExceeded" => Subscription,
            "unauthorized" => Authentication,
            "rateLimitExceeded" | "flexUnavailable" | "serverOverloaded" => Capacity,
            "contextWindowExceeded"
            | "sessionBudgetExceeded"
            | "cyberPolicy"
            | "misalignmentPolicyViolation"
            | "tooManyDenials"
            | "internalServerError"
            | "badRequest"
            | "threadRollbackFailed"
            | "sandboxError"
            | "other" => Other,
            _ => Unclassified,
        });
    }
    let o = v
        .as_object()
        .context("Codex error must be string or object")?;
    let tags = [
        "httpConnectionFailed",
        "responseStreamConnectionFailed",
        "responseStreamDisconnected",
        "responseTooManyFailedAttempts",
        "activeTurnNotSteerable",
    ];
    let present = tags
        .iter()
        .filter(|k| o.contains_key(**k))
        .collect::<Vec<_>>();
    if present.is_empty() {
        return Ok(Unclassified);
    }
    ensure!(
        present.len() == 1 && o.len() == 1,
        "ambiguous Codex error key-tag"
    );
    let tag = **present.first().unwrap();
    record(
        &v[tag],
        if tag == "activeTurnNotSteerable" {
            "turnKind:review|compact"
        } else {
            "?httpStatusCode:u16~"
        },
        false,
        &CodexWireContract::default(),
    )?;
    if tag != "activeTurnNotSteerable"
        && matches!(v[tag]["httpStatusCode"].as_u64(), Some(429 | 503))
    {
        Ok(Capacity)
    } else {
        Ok(Other)
    }
}
