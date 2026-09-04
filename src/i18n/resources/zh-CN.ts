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
      updateUnavailable: "界面设置读取完成前无法保存更改。",
      updateFailed: "界面设置保存失败，已恢复上次保存的外观。",
      nativeThemeFailed: "窗口主题同步失败，界面内容仍可继续使用。",
    },
  },
  shell: {
    routes: {
      overview: "概览",
      lexicons: "码表",
      phrases: "短语",
      lookup: "反查",
      radicals: "字根",
      learning: "学习",
      settings: "设置",
    },
    navigation: {
      emptyPath: "空路径",
      unknownPath: "无法识别内部路径“{{path}}”，已返回概览。",
    },
    sidebar: {
      label: "主导航",
      collapse: "折叠侧栏",
      expand: "展开侧栏",
      developing: "开发中",
    },
    status: {
      loading: "正在准备应用",
      ready: "应用已就绪",
    },
    placeholder: {
      state: "功能暂未完善",
      milestone: "计划阶段 {{milestone}}",
      planned: "已列入后续任务",
    },
    featureGate: {
      loadingTitle: "正在读取功能状态",
      loadingDetail: "正在确认当前构建包含的能力。",
      errorTitle: "功能状态暂时不可用",
      missingFeature: "功能目录缺少 {{feature}}，已按不可用处理。",
    },
    lexicons: {
      title: "码表管理",
      detail: "查看和维护五笔码表的统一入口。",
      placeholderTitle: "码表读取与管理",
      placeholderDetail: "后续阶段将提供码表浏览、筛选和编辑能力。",
      readyTitle: "码表入口已可用",
      readyDetail:
        "当前构建包含码表读取能力，具体工作区将在对应领域任务中接入。",
    },
    phrases: {
      title: "短语管理",
      detail: "管理用户短语和词组的统一入口。",
      placeholderTitle: "短语读取与管理",
      placeholderDetail: "后续阶段将提供短语浏览、编辑和整理能力。",
      readyTitle: "短语入口已可用",
      readyDetail:
        "当前构建包含短语读取能力，具体工作区将在对应领域任务中接入。",
    },
    lookup: {
      title: "编码反查",
      detail: "按文字或编码查找五笔信息的统一入口。",
      placeholderTitle: "五笔编码反查",
      placeholderDetail: "后续阶段将提供文字、编码和候选结果查询。",
      readyTitle: "反查入口已可用",
      readyDetail: "当前构建包含反查能力，具体查询界面将在对应领域任务中接入。",
    },
    radicals: {
      title: "字根参考",
      detail: "查询字根分区和拆分资料的统一入口。",
      placeholderTitle: "字根参考资料",
      placeholderDetail: "后续阶段将提供字根分区、键位和拆分参考。",
      readyTitle: "字根入口已可用",
      readyDetail: "当前构建包含字根参考能力，具体内容将在对应领域任务中接入。",
    },
    learning: {
      title: "学习记录",
      detail: "查看练习与自学习状态的统一入口。",
      placeholderTitle: "自学习与练习",
      placeholderDetail: "后续阶段将提供练习、反馈和学习记录。",
      readyTitle: "学习入口已可用",
      readyDetail: "当前构建包含自学习能力，具体工作区将在对应领域任务中接入。",
    },
    settings: {
      title: "应用设置",
      detail: "外观设置即时生效，其余分组会随对应能力逐步开放。",
      groups: {
        ime: {
          title: "输入法",
          detail: "输入法注册、启停与系统状态由系统集成阶段提供。",
        },
        wubi: {
          title: "五笔行为",
          detail: "编码、候选和输入行为会随码表能力一并提供。",
        },
        candidate: {
          title: "候选窗口",
          detail: "候选窗口外观与交互由后续输入体验任务负责。",
        },
        shortcuts: {
          title: "快捷键",
          detail: "快捷键注册、录制和冲突检查由下一项应用动作任务负责。",
        },
        appearance: {
          title: "外观",
          detail: "更改会即时应用并保存到应用配置。",
        },
        network: {
          title: "网络",
          detail: "资源更新和网络访问会在资源同步能力可用后提供。",
        },
        data: {
          title: "数据",
          detail: "导入、导出、备份和迁移会在对应数据任务中提供。",
        },
        about: {
          title: "关于",
          detail: "版本详情、许可和诊断导出由后续应用信息任务负责。",
        },
      },
      appearance: {
        theme: "主题",
        density: "界面密度",
        sidebar: "折叠侧栏",
        sidebarDetail: "侧栏和这里使用同一项已保存的界面设置。",
        themeOptions: {
          system: "跟随系统",
          light: "浅色",
          dark: "深色",
        },
        densityOptions: {
          standard: "标准",
          compact: "紧凑",
        },
      },
    },
  },
} as const;
