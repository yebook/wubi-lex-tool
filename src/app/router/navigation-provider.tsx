import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { ReactNode } from "react";
import { useLocation, useNavigate, useNavigationType } from "react-router";

import { useOverlayRoot } from "../../components/ui/overlay-provider";
import type { CanonicalRoutePath } from "./catalog";
import { validateProductPath } from "./path";

interface HistoryState {
  entries: string[];
  index: number;
}

interface NavigationContextValue {
  canGoBack: boolean;
  warning: string | null;
  clearWarning(): void;
  goBack(): void;
  navigateProductPath(path: string): void;
  rememberFocus(element: HTMLElement): void;
}

interface NavigationProviderProps {
  children: ReactNode;
  initialWarning?: string | null;
}

const NavigationContext = createContext<NavigationContextValue | null>(null);

export function NavigationProvider({
  children,
  initialWarning = null,
}: NavigationProviderProps) {
  const location = useLocation();
  const navigationType = useNavigationType();
  const navigate = useNavigate();
  const overlayRoot = useOverlayRoot();
  const history = useRef<HistoryState>({
    entries: [location.key],
    index: 0,
  });
  const processedKey = useRef(location.key);
  const previousPath = useRef(location.pathname);
  const warningPath = useRef<string | null>(
    initialWarning ? location.pathname : null,
  );
  const focusRecords = useRef(new Map<string, HTMLElement>());
  const [canGoBack, setCanGoBack] = useState(false);
  const [warning, setWarning] = useState<string | null>(initialWarning);

  useEffect(() => {
    if (previousPath.current === location.pathname) {
      return;
    }
    previousPath.current = location.pathname;
    if (warningPath.current !== location.pathname) {
      warningPath.current = null;
      setWarning(null);
    }
  }, [location.pathname]);

  const focusRouteHeading = useCallback(() => {
    const heading = document.querySelector<HTMLElement>(
      "h1[data-route-heading]",
    );
    const scrollContainer = heading?.closest<HTMLElement>(".shell-route-main");
    if (scrollContainer) {
      scrollContainer.scrollTop = 0;
    }
    heading?.focus({ preventScroll: true });
  }, []);

  useEffect(() => {
    if (processedKey.current === location.key) {
      const timer = window.setTimeout(focusRouteHeading, 0);
      return () => window.clearTimeout(timer);
    }

    const current = history.current;
    let shouldRestore = false;
    if (navigationType === "PUSH") {
      current.entries = current.entries.slice(0, current.index + 1);
      current.entries.push(location.key);
      current.index = current.entries.length - 1;
    } else if (navigationType === "REPLACE") {
      current.entries[current.index] = location.key;
    } else {
      const targetIndex = current.entries.indexOf(location.key);
      if (targetIndex === -1) {
        current.entries = [location.key];
        current.index = 0;
      } else {
        current.index = targetIndex;
        shouldRestore = true;
      }
    }
    processedKey.current = location.key;
    setCanGoBack(current.index > 0);

    const timer = window.setTimeout(() => {
      const trigger = shouldRestore
        ? focusRecords.current.get(location.key)
        : undefined;
      if (trigger && document.contains(trigger)) {
        trigger.focus({ preventScroll: true });
      } else {
        focusRouteHeading();
      }
    }, 0);
    return () => window.clearTimeout(timer);
  }, [focusRouteHeading, location.key, navigationType]);

  const goBack = useCallback(() => {
    if (history.current.index > 0) {
      void navigate(-1);
    }
  }, [navigate]);

  const navigateProductPath = useCallback(
    (path: string) => {
      const result = validateProductPath(path);
      warningPath.current = result.warning ? result.path : null;
      setWarning(result.warning);
      if (result.path === location.pathname) {
        return;
      }
      void navigate(result.path, {
        replace: result.kind === "redirect",
      });
    },
    [location.pathname, navigate],
  );

  const rememberFocus = useCallback(
    (element: HTMLElement) => {
      focusRecords.current.set(location.key, element);
    },
    [location.key],
  );

  const clearWarning = useCallback(() => {
    warningPath.current = null;
    setWarning(null);
  }, []);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        event.key === "ArrowLeft" &&
        event.altKey &&
        !event.ctrlKey &&
        !event.metaKey &&
        !event.shiftKey
      ) {
        event.preventDefault();
        goBack();
        return;
      }

      if (
        event.key !== "Escape" ||
        event.defaultPrevented ||
        event.altKey ||
        event.ctrlKey ||
        event.metaKey ||
        event.shiftKey ||
        history.current.index === 0 ||
        isEditableTarget(event.target) ||
        Boolean(overlayRoot?.childElementCount)
      ) {
        return;
      }
      event.preventDefault();
      goBack();
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [goBack, overlayRoot]);

  const value = useMemo<NavigationContextValue>(
    () => ({
      canGoBack,
      warning,
      clearWarning,
      goBack,
      navigateProductPath,
      rememberFocus,
    }),
    [
      canGoBack,
      clearWarning,
      goBack,
      navigateProductPath,
      rememberFocus,
      warning,
    ],
  );

  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  );
}

export function useAppNavigation(): NavigationContextValue {
  const value = useContext(NavigationContext);
  if (!value) {
    throw new Error("useAppNavigation must be used within NavigationProvider");
  }
  return value;
}

export function useCanonicalRoutePath(): CanonicalRoutePath | null {
  const path = useLocation().pathname;
  const result = validateProductPath(path);
  return result.kind === "canonical" ? result.path : null;
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) {
    return false;
  }
  return Boolean(
    target.closest(
      "input, textarea, select, [contenteditable]:not([contenteditable='false'])",
    ),
  );
}
