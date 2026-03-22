// src/parser.ts
function parseSom(input) {
  const obj = typeof input === "string" ? JSON.parse(input) : input;
  if (!isValidSom(obj)) {
    throw new Error("Invalid SOM: missing required fields (som_version, url, title, regions, meta)");
  }
  return obj;
}
function isValidSom(input) {
  if (input == null || typeof input !== "object") return false;
  const o = input;
  if (typeof o.som_version !== "string") return false;
  if (typeof o.url !== "string") return false;
  if (typeof o.title !== "string") return false;
  if (!Array.isArray(o.regions)) return false;
  if (o.meta == null || typeof o.meta !== "object") return false;
  return true;
}
function fromPlasmate(jsonOutput) {
  try {
    return parseSom(jsonOutput);
  } catch {
    const start = jsonOutput.indexOf("{");
    const end = jsonOutput.lastIndexOf("}");
    if (start === -1 || end === -1 || end <= start) {
      throw new Error("No JSON object found in Plasmate output");
    }
    return parseSom(jsonOutput.slice(start, end + 1));
  }
}

// src/query.ts
function collectElements(elements) {
  const result = [];
  for (const el of elements) {
    result.push(el);
    if (el.children) {
      result.push(...collectElements(el.children));
    }
  }
  return result;
}
function getAllElements(som) {
  const result = [];
  for (const region of som.regions) {
    result.push(...collectElements(region.elements));
  }
  return result;
}
function findByRole(som, role) {
  return getAllElements(som).filter((el) => el.role === role);
}
function findById(som, id) {
  return getAllElements(som).find((el) => el.id === id);
}
function findByText(som, text, options) {
  const all = getAllElements(som);
  if (options?.exact) {
    return all.filter((el) => el.text === text || el.label === text);
  }
  const lower = text.toLowerCase();
  return all.filter(
    (el) => el.text && el.text.toLowerCase().includes(lower) || el.label && el.label.toLowerCase().includes(lower)
  );
}
function getInteractiveElements(som) {
  return getAllElements(som).filter((el) => el.actions && el.actions.length > 0);
}
function getLinks(som) {
  return findByRole(som, "link").filter((el) => el.attrs?.href).map((el) => ({
    text: el.text ?? "",
    href: el.attrs.href,
    id: el.id
  }));
}
function getForms(som) {
  return som.regions.filter((r) => r.role === "form");
}
var INPUT_ROLES = ["text_input", "textarea", "select", "checkbox", "radio"];
function getInputs(som) {
  return getAllElements(som).filter((el) => INPUT_ROLES.includes(el.role));
}
function getHeadings(som) {
  return findByRole(som, "heading").map((el) => ({
    level: el.attrs?.level ?? 1,
    text: el.text ?? "",
    id: el.id
  }));
}
function getText(som) {
  return getAllElements(som).map((el) => el.text ?? el.label ?? "").filter(Boolean).join("\n");
}
function getTextByRegion(som) {
  return som.regions.map((r) => ({
    region: r.id,
    role: r.role,
    text: collectElements(r.elements).map((el) => el.text ?? el.label ?? "").filter(Boolean).join("\n")
  }));
}
function getCompressionRatio(som) {
  if (!som.meta?.som_bytes || som.meta.som_bytes === 0) return 0;
  return som.meta.html_bytes / som.meta.som_bytes;
}
function toMarkdown(som) {
  const lines = [];
  if (som.title) {
    lines.push(`# ${som.title}`);
    lines.push("");
  }
  for (const region of som.regions) {
    if (region.role === "form") {
      const action = region.action ? ` (${region.method ?? "POST"} ${region.action})` : "";
      lines.push(`### Form${action}`);
      lines.push("");
      for (const el of collectElements(region.elements)) {
        if (INPUT_ROLES.includes(el.role)) {
          const label = el.label ?? el.attrs?.placeholder ?? el.role;
          lines.push(`- **${label}** (${el.role})`);
        } else if (el.role === "button") {
          lines.push(`- [${el.text ?? "Button"}] (button)`);
        }
      }
      lines.push("");
      continue;
    }
    for (const el of collectElements(region.elements)) {
      switch (el.role) {
        case "heading": {
          const level = el.attrs?.level ?? 1;
          lines.push(`${"#".repeat(Math.min(level + 1, 6))} ${el.text ?? ""}`);
          lines.push("");
          break;
        }
        case "paragraph":
          if (el.text) {
            lines.push(el.text);
            lines.push("");
          }
          break;
        case "link":
          lines.push(`- [${el.text ?? ""}](${el.attrs?.href ?? "#"})`);
          break;
        case "image":
          lines.push(`![${el.attrs?.alt ?? ""}](${el.attrs?.src ?? ""})`);
          lines.push("");
          break;
        case "list": {
          const items = el.attrs?.items ?? [];
          for (const item of items) {
            lines.push(`- ${item.text}`);
          }
          if (items.length) lines.push("");
          break;
        }
        default:
          if (el.text) {
            lines.push(el.text);
            lines.push("");
          }
      }
    }
  }
  return lines.join("\n").trim() + "\n";
}
function filter(som, predicate) {
  return getAllElements(som).filter(predicate);
}
export {
  filter,
  findById,
  findByRole,
  findByText,
  fromPlasmate,
  getAllElements,
  getCompressionRatio,
  getForms,
  getHeadings,
  getInputs,
  getInteractiveElements,
  getLinks,
  getText,
  getTextByRegion,
  isValidSom,
  parseSom,
  toMarkdown
};
