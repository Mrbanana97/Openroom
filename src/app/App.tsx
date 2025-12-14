import { AdjustmentPanel } from "../components/AdjustmentPanel";
import { Filmstrip } from "../components/Filmstrip";
import { HeaderBar } from "../components/HeaderBar";
import { SidebarPanel } from "../components/layout/SidebarPanel";
import { PresetList } from "../components/PresetList";
import { PreviewStage } from "../components/PreviewStage";
import { RecipeSyncBoundary } from "./RecipeSyncBoundary";
import { SettingsCard } from "../components/SettingsCard";

function App() {
  return (
    <div className="flex min-h-screen flex-col bg-[var(--bg)] text-[var(--text-primary)]">
      <RecipeSyncBoundary />
      <HeaderBar />
      <main className="mx-auto flex w-full max-w-screen-2xl flex-1 flex-col gap-3 px-4 py-3">
        <div className="flex min-h-0 flex-1 gap-3">
          <SidebarPanel title="Presets" width={240}>
            <PresetList />
          </SidebarPanel>

          <div className="flex min-w-0 flex-1 flex-col gap-3">
            <PreviewStage />
            <Filmstrip />
            <SettingsCard className="mt-2" />
          </div>

          <SidebarPanel title="Adjustments" width={320}>
            <AdjustmentPanel />
          </SidebarPanel>
        </div>
      </main>
    </div>
  );
}

export default App;
