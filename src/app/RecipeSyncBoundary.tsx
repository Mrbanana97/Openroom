import { useEffect } from "react";
import { useRecipeSync } from "../features/editor/useRecipeSync";
import { useSelectedAsset } from "../features/library/store";

export function RecipeSyncBoundary() {
  const asset = useSelectedAsset();
  useRecipeSync(asset?.id);

  // no UI rendering; just synchronizes recipes for current asset
  useEffect(() => {}, [asset?.id]);
  return null;
}
