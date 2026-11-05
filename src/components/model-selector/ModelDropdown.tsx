import React from "react";
import { useTranslation } from "react-i18next";
import { CloudDownload } from "lucide-react";
import type { ModelInfo } from "@/bindings";
import { useModelStore } from "@/stores/modelStore";
import {
  getTranslatedModelName,
  getTranslatedModelDescription,
} from "../../lib/utils/modelTranslation";

interface ModelDropdownProps {
  models: ModelInfo[];
  currentModelId: string;
  // A file is transcribing: show the models, but don't let them be picked.
  locked?: boolean;
  onModelSelect: (modelId: string) => void;
}

const ModelDropdown: React.FC<ModelDropdownProps> = ({
  models,
  currentModelId,
  locked = false,
  onModelSelect,
}) => {
  const { t } = useTranslation();
  const checking = useModelStore((s) => s.isCheckingUpdates);
  const newCount = useModelStore((s) => s.newModelIds.length);
  const downloadedModels = models.filter((m) => m.is_downloaded);

  const handleModelClick = (modelId: string) => {
    if (locked) return;
    onModelSelect(modelId);
  };

  return (
    <div className="absolute bottom-full start-0 mb-2 w-64 max-h-[60vh] overflow-y-auto bg-background border border-line rounded-lg shadow-lg py-2 z-50">
      <div className="flex items-center justify-between gap-2 px-3 pb-2 mb-1 border-b border-line">
        <span className="text-xs text-ink-3">
          {newCount > 0
            ? t("modelUpdate.newAvailable", { count: newCount })
            : t("settings.models.update.short")}
        </span>
        <button
          type="button"
          onClick={() => useModelStore.getState().checkModelUpdates()}
          disabled={checking}
          aria-label={t("settings.models.update.tooltip")}
          title={t("settings.models.update.tooltip")}
          className="grid size-[22px] cursor-pointer place-items-center rounded-full text-ink-3 hover:bg-wash hover:text-text disabled:cursor-default disabled:opacity-60 focus:outline-none focus-visible:ring-1 focus-visible:ring-logo-primary"
        >
          <CloudDownload
            className={`size-[13px] ${checking ? "animate-pulse" : ""}`}
            aria-hidden="true"
          />
        </button>
      </div>
      {locked && (
        <div className="px-3 pb-2 mb-1 text-xs text-ink-2 border-b border-line">
          {t("modelSelector.lockedWhileTranscribing")}
        </div>
      )}
      {downloadedModels.length > 0 ? (
        <div>
          {downloadedModels.map((model) => (
            <div
              key={model.id}
              onClick={() => handleModelClick(model.id)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  handleModelClick(model.id);
                }
              }}
              tabIndex={locked ? -1 : 0}
              role="button"
              aria-disabled={locked}
              className={`w-full px-3 py-2 text-start transition-colors focus:outline-none ${
                locked
                  ? "opacity-50 cursor-not-allowed"
                  : "hover:bg-wash cursor-pointer"
              } ${
                currentModelId === model.id
                  ? "bg-logo-primary/10 text-logo-primary"
                  : ""
              }`}
            >
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm text-text">
                    {getTranslatedModelName(model, t)}
                    {model.is_custom && (
                      <span className="ms-1.5 text-[10px] font-medium text-ink-3 uppercase">
                        {t("modelSelector.custom")}
                      </span>
                    )}
                    {model.supports_streaming && (
                      <span className="ms-1.5 text-[10px] font-medium text-logo-primary uppercase">
                        {t("modelSelector.streaming")}
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-ink-3 italic pe-4">
                    {getTranslatedModelDescription(model, t)}
                  </div>
                </div>
                {currentModelId === model.id && (
                  <div className="text-xs text-logo-primary">
                    {t("modelSelector.active")}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="px-3 py-2 text-sm text-ink-2">
          {t("modelSelector.noModelsAvailable")}
        </div>
      )}
    </div>
  );
};

export default ModelDropdown;
