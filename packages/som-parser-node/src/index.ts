// Types
export type {
  Som,
  SomElement,
  SomRegion,
  SomMeta,
  SomElementAttrs,
  RegionRole,
  ElementRole,
  ElementAction,
  SemanticHint,
  SelectOption,
  ListItem,
  LinkElement,
  StructuredData,
  AriaState,
  SomShadowRoot,
} from './types.js';

// Parser
export { parseSom, isValidSom, fromPlasmate } from './parser.js';

// Query utilities
export {
  getAllElements,
  findActionTargetByCacheKey,
  findActionTargetByHtmlId,
  findActionTargetById,
  findByAction,
  findByHint,
  findByRole,
  findById,
  findByHtmlId,
  findByText,
  getActionPlan,
  getActionPlanCacheKey,
  getActionPlanFingerprint,
  getActionPlanIndex,
  getActionPlanSummary,
  getEnabledActionPlan,
  getInteractiveElements,
  getLinks,
  getForms,
  getInputs,
  getHeadings,
  getText,
  getTextByRegion,
  getCompressionRatio,
  toMarkdown,
  filter,
} from './query.js';
