; plainmark NSIS 安装钩子
;
; 作用：为 .md / .markdown 注册资源管理器右键「新建」菜单项。
;
; 为什么需要钩子：tauri.conf.json 的 bundle.fileAssociations 只写文件关联
; （双击打开、图标、「类型」列名称），**不写** ShellNew 键。而右键「新建」
; 菜单正是由 ShellNew 驱动的，所以必须在这里补上。
;
; 时机：用 POSTINSTALL，此时 Tauri 已写好 Software\Classes\.md 父键。
;
; 键位：与 tauri.conf.json 的 installMode = "currentUser" 对应，写 HKCU。
; 若将来改成 perMachine，这里必须相应改为 HKLM。

!macro NSIS_HOOK_POSTINSTALL
  ; NullFile 表示新建时创建一个 0 字节的空文件——与记事本新建文本文档一致
  WriteRegStr HKCU "Software\Classes\.md\ShellNew" "NullFile" ""
  WriteRegStr HKCU "Software\Classes\.markdown\ShellNew" "NullFile" ""

  ; 让资源管理器「新建」菜单的条目显示为「Markdown 文档」而非「.md 文件」。
  ; 若该扩展名已被其它程序占用，此处不覆盖其默认值，只补充 ShellNew。
  DetailPrint "已注册 Markdown 文档的新建菜单项"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; 卸载时清理，避免留下孤儿键导致「新建」菜单出现无效项
  DeleteRegKey HKCU "Software\Classes\.md\ShellNew"
  DeleteRegKey HKCU "Software\Classes\.markdown\ShellNew"
!macroend
