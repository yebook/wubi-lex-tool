export const zhCN = {
  common: {
    retry: "重新读取",
  },
  window: {
    controls: "窗口控制",
    minimizeToTray: "最小化到托盘",
    maximize: "最大化窗口",
    restore: "还原窗口",
    close: "关闭窗口",
    warning: {
      stateListener: "窗口状态实时监听暂时不可用。",
      noticeListener: "窗口告警实时监听暂时不可用。",
      stateSnapshot: "窗口状态暂时不可用。",
      command: "窗口操作未能完成。",
    },
  },
  runtime: {
    eyebrow: "应用外壳",
    title: "运行状态",
    connected: "已连接本地运行时",
    loading: {
      title: "正在读取运行时状态",
      detail: "正在检查进程权限、会话标记和启动请求。",
    },
    loadError: {
      title: "无法连接本地运行时",
      fallback: "无法读取本地运行时状态。",
    },
    warning: {
      refresh: "启动请求已收到，但完整运行状态暂时无法刷新。可稍后重新读取。",
      listener: "实时启动监听不可用；重新打开窗口可刷新完整状态。",
      restricted: "实时监听受限",
    },
    summary: {
      label: "运行时摘要",
      privilege: "进程权限",
      recovery: "会话检查",
      launch: "最近启动",
    },
    privilege: {
      elevated: {
        label: "已获得管理员权限",
        detail: "当前进程令牌已通过系统检查。",
      },
      notElevated: {
        label: "未以管理员身份运行",
        detail:
          "请关闭应用并以管理员身份重新启动；获得权限前不会执行系统写入。",
      },
      unavailable: {
        label: "权限状态未知",
        evidence: "{{stage}}，系统代码 {{code}}",
        noEvidence: "未返回系统诊断信息",
      },
    },
    recovery: {
      clean: {
        label: "未发现异常会话",
        detail: "当前启动前没有遗留的会话标记。",
      },
      abnormal: {
        label: "发现 {{count}} 个异常会话标记",
        detail: "这只表示应用上次未正常退出，暂未执行系统恢复。",
      },
    },
    launch: {
      fallback: {
        label: "已回退为普通启动",
        detail: "启动参数存在问题，请查看下方警告。",
      },
      hidden: {
        label: "后台启动",
        detail: "窗口按 /tray 请求创建为隐藏状态。",
      },
      navigation: {
        label: "带导航目标启动",
        detail: "目标已通过传输校验，将由后续路由层处理。",
      },
      normal: {
        label: "普通启动",
        detail: "窗口按默认方式显示。",
      },
    },
    request: {
      eyebrow: "最近请求",
      title: "启动参数",
      primary: "主实例",
      secondary: "第二实例",
      windowMode: "窗口模式",
      hidden: "隐藏",
      visible: "可见",
      navigation: "内部导航",
      unspecified: "未指定",
      warnings: "参数警告",
    },
    notices: {
      eyebrow: "诊断",
      title: "可见警告",
      empty: "当前没有需要处理的运行时警告。",
    },
    featureCatalogFallback: "无法读取应用功能目录。",
  },
  ui: {
    dialogClose: "关闭对话框",
    preferences: {
      listenerFailed: "界面设置实时同步暂时不可用。",
      snapshotFailed: "界面设置暂时无法读取，已使用默认外观。",
      updateFailed: "界面设置保存失败，已恢复上次保存的外观。",
      nativeThemeFailed: "窗口主题同步失败，界面内容仍可继续使用。",
    },
  },
} as const;
