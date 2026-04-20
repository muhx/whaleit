// AI provider type definitions

export interface ModelCapabilities {
  tools: boolean;
  thinking: boolean;
  vision: boolean;
  /** Whether the model supports streaming responses. */
  streaming: boolean;
}

export interface ModelCapabilityOverrides {
  tools?: boolean;
  thinking?: boolean;
  vision?: boolean;
  streaming?: boolean;
}

export interface MergedModel {
  id: string;
  /** Display name (may differ from id for fetched models). */
  name?: string;
  capabilities: ModelCapabilities;
  /** Whether this model is from the catalog (true) or dynamically fetched (false). */
  isCatalog: boolean;
  /** Whether this model is marked as a user favorite. */
  isFavorite: boolean;
  /** Whether capabilities have user overrides applied. */
  hasCapabilityOverrides: boolean;
}

export interface ConnectionField {
  key: string;
  label: string;
  type: string;
  placeholder: string;
  required: boolean;
  helpUrl?: string;
}

export interface CapabilityInfo {
  name: string;
  description: string;
  icon: string;
}

export interface ProviderTuning {
  /** Sampling temperature. Lower values → more deterministic output. */
  temperature?: number;
  /** Maximum output tokens per response (safety cap). */
  maxTokens?: number;
  /** Max tokens when the model's thinking/reasoning mode is enabled. */
  maxTokensThinking?: number;
  /**
   * Provider-specific raw JSON (Ollama's `num_ctx`/`repeat_penalty`, Gemini's
   * `safetySettings`, etc.). Catalog-only — not user-editable.
   */
  extraOptions?: Record<string, unknown>;
}

export interface ProviderTuningOverrides {
  temperature?: number;
  maxTokens?: number;
  maxTokensThinking?: number;
  extraOptionOverrides?: Record<string, number | boolean | string | null>;
}

export interface MergedProvider {
  // From catalog (immutable)
  id: string;
  name: string;
  type: string;
  icon: string;
  description: string;
  envKey: string;
  connectionFields: ConnectionField[];
  models: MergedModel[];
  defaultModel: string;
  documentationUrl: string;

  // From user settings (mutable)
  enabled: boolean;
  favorite: boolean;
  selectedModel?: string;
  customUrl?: string;
  priority: number;
  /** User's favorite model IDs (including fetched models not in catalog). */
  favoriteModels: string[];
  /** Capability overrides for specific models. */
  modelCapabilityOverrides: Record<string, ModelCapabilityOverrides>;
  /** Allowlist of tool IDs that this provider can use. null = all tools enabled. */
  toolsAllowlist?: string[] | null;

  // Computed
  hasApiKey: boolean;
  isDefault: boolean;
  /** Whether this provider supports dynamic model listing via API. */
  supportsModelListing: boolean;

  // Tuning (three views: what ships, what user changed, what runtime uses)
  /** Catalog tuning defaults for this provider (immutable reference). */
  catalogTuning?: ProviderTuning;
  /** User-supplied overrides; undefined means the user hasn't customized. */
  tuningOverrides?: ProviderTuningOverrides;
  /** Effective tuning the runtime will use (catalog merged with overrides). */
  resolvedTuning?: ProviderTuning;
}

export interface AiProvidersResponse {
  providers: MergedProvider[];
  capabilities: Record<string, CapabilityInfo>;
  defaultProvider?: string;
}

export interface ModelCapabilityOverrideUpdate {
  /** The model ID to update. */
  modelId: string;
  /** The capability overrides to set. Use undefined to remove overrides for this model. */
  overrides?: ModelCapabilityOverrides;
}

export interface UpdateProviderSettingsRequest {
  providerId: string;
  enabled?: boolean;
  favorite?: boolean;
  selectedModel?: string;
  customUrl?: string;
  priority?: number;
  /** Set capability overrides for a specific model. */
  modelCapabilityOverride?: ModelCapabilityOverrideUpdate;
  /** Update the list of favorite models (replaces the entire list). */
  favoriteModels?: string[];
  /** Update tools allowlist. null = all tools enabled, [] = no tools, [...] = only specified tools. */
  toolsAllowlist?: string[] | null;
  /** Update user tuning overrides. null = reset to catalog defaults, {} or partial = set. */
  tuningOverrides?: ProviderTuningOverrides | null;
}

export interface SetDefaultProviderRequest {
  providerId?: string;
}

export interface FetchedModel {
  id: string;
  name?: string;
}

export interface ListModelsResponse {
  models: FetchedModel[];
  supportsListing: boolean;
}
