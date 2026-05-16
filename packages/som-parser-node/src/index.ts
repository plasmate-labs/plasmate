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
  findByAction,
  findActionTarget,
  findActionTargetByCacheKey,
  findActionTargetByHtmlId,
  findActionTargetById,
  findActionTargetByLabel,
  findActionTargetByTestId,
  findActionTargetsByLabel,
  findByHint,
  findByRole,
  findById,
  findByHtmlId,
  findByLabel,
  findByText,
  getActionPlan,
  getActionPlanCacheKey,
  getActionPlanIndex,
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
