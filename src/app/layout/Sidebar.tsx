import { useTranslation } from "react-i18next";
import { NavLink, useLocation } from "react-router";
import { useStore } from "zustand";

import {
  Button,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "../../components/ui";
import { PanelLeftClose, PanelLeftOpen } from "../../icons/ui";
import { cn } from "../../lib/cn";
import { featuresStore } from "../../stores/features";
import { useUiPreferences } from "../providers/ui-preferences-provider";
import { routeCatalog } from "../router/catalog";
import { useAppNavigation } from "../router/navigation-provider";

export function Sidebar() {
  const { t } = useTranslation("shell");
  const location = useLocation();
  const { status, ui, setSidebarCollapsed } = useUiPreferences();
  const navigation = useAppNavigation();
  const featureCatalog = useStore(featuresStore, (state) => state.catalog);
  const featureStatus = useStore(featuresStore, (state) => state.status);
  const collapsed = ui.sidebarCollapsed;
  const collapseLabel = collapsed ? t("sidebar.expand") : t("sidebar.collapse");
  const CollapseIcon = collapsed ? PanelLeftOpen : PanelLeftClose;

  return (
    <aside className="shell-sidebar" data-collapsed={collapsed || undefined}>
      <nav className="sidebar-nav" aria-label={t("sidebar.label")}>
        {routeCatalog.map((route) => {
          const Icon = route.icon;
          const label = t(route.labelKey);
          const link = (
            <NavLink
              key={route.id}
              to={route.path}
              aria-label={label}
              className={({ isActive }) =>
                cn(
                  "sidebar-link",
                  isActive && "sidebar-link-active",
                  route.id === "settings" && "sidebar-link-settings",
                )
              }
              onClick={(event) => {
                navigation.rememberFocus(event.currentTarget);
                navigation.clearWarning();
                if (route.path === location.pathname) {
                  event.preventDefault();
                }
              }}
            >
              <Icon aria-hidden="true" strokeWidth={1.8} />
              <span className="sidebar-link-label">{label}</span>
              {featureStatus === "ready" &&
              "feature" in route &&
              !featureCatalog.features.some(
                (feature) => feature.id === route.feature && feature.available,
              ) ? (
                <span className="sidebar-development-state">
                  {t("sidebar.developing")}
                </span>
              ) : null}
            </NavLink>
          );

          return collapsed ? (
            <Tooltip key={route.id}>
              <TooltipTrigger asChild>{link}</TooltipTrigger>
              <TooltipContent side="right">{label}</TooltipContent>
            </Tooltip>
          ) : (
            link
          );
        })}
      </nav>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            className="sidebar-collapse-button"
            variant="ghost"
            size="icon"
            aria-label={collapseLabel}
            aria-expanded={!collapsed}
            disabled={status !== "ready"}
            onClick={() => void setSidebarCollapsed(!collapsed)}
          >
            <CollapseIcon aria-hidden="true" strokeWidth={1.8} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">{collapseLabel}</TooltipContent>
      </Tooltip>
    </aside>
  );
}
