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
  ShadowRoot,
} from './types.js';

// Parser
export { parseSom, isValidSom, fromPlasmate } from './parser.js';

// Query utilities
export {
  getAllElements,
  findByRole,
  findById,
  findByText,
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
