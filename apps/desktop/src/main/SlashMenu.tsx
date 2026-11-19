import { SuggestionMenu } from "@blocknote/core/extensions";
import {
  type DefaultReactSuggestionItem,
  type SuggestionMenuProps,
  useComponentsContext,
  useExtensionState,
} from "@blocknote/react";
import { Search } from "lucide-react";
import { useMemo } from "react";

/**
 * The "/" command menu with a search field on top. The query is typed in the note right after
 * "/" (so arrow keys and Enter keep working without moving focus); the field shows it and
 * the list below is filtered by it.
 */
export function SlashMenu(props: SuggestionMenuProps<DefaultReactSuggestionItem>) {
  const components = useComponentsContext();
  const query = useExtensionState(SuggestionMenu, {
    selector: (state) => state?.query ?? "",
  });
  const { items, selectedIndex, onItemClick, loadingState } = props;

  const rows = useMemo(() => {
    if (!components) return [];
    const out: React.ReactNode[] = [];
    let group: string | undefined;
    items.forEach((item, i) => {
      if (item.group !== group) {
        group = item.group;
        out.push(
          <components.SuggestionMenu.Label
            key={`group-${group}`}
            className="bn-suggestion-menu-label"
          >
            {group}
          </components.SuggestionMenu.Label>,
        );
      }
      out.push(
        <components.SuggestionMenu.Item
          key={item.title}
          className="bn-suggestion-menu-item"
          item={item}
          id={`bn-suggestion-menu-item-${i}`}
          isSelected={i === selectedIndex}
          onClick={() => onItemClick?.(item)}
        />,
      );
    });
    return out;
  }, [components, items, selectedIndex, onItemClick]);

  if (!components) return null;
  return (
    <components.SuggestionMenu.Root
      id="bn-suggestion-menu"
      className="bn-suggestion-menu slash-menu"
    >
      <div className="slash-search" aria-hidden="true">
        <Search size={14} />
        {query ? (
          <span className="slash-query">
            {query}
            <span className="slash-caret" />
          </span>
        ) : (
          <span className="slash-placeholder">
            <span className="slash-caret" />
            Search commands…
          </span>
        )}
      </div>
      {rows}
      {rows.length === 0 && loadingState === "loaded" && (
        <components.SuggestionMenu.EmptyItem className="bn-suggestion-menu-item">
          No commands match “{query}”
        </components.SuggestionMenu.EmptyItem>
      )}
    </components.SuggestionMenu.Root>
  );
}
