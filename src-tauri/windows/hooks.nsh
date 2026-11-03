; plainmark NSIS 安装钩子
;
; 作用：为 .md 注册资源管理器右键「新建」菜单项，并清理旧版本留下的重复注册。
;
; 为什么需要钩子：tauri.conf.json 的 bundle.fileAssociations 只写文件关联
; （双击打开、图标、「类型」列名称），**不写** ShellNew 键。而右键「新建」
; 菜单正是由 ShellNew 驱动的，所以必须在这里补上。
;
; 为什么只注册 .md 而不注册 .markdown：
; 「新建」菜单是按扩展名逐条列出的，显示名取自该扩展名的文件类型名。
; .md 与 .markdown 共用「Markdown 文档」这一个类型名，两个都注册就会
; 在菜单里出现两条一模一样的条目。记事本同样只给 .txt 注册、不给 .text。
; .markdown 文件仍可通过文件关联双击打开，只是不出现在「新建」菜单。
;
; 时机：用 POSTINSTALL。它在 Tauri 写完文件关联之后执行（NSIS 模板中关联写入
; 在前、本钩子在后），且在**每次**安装时都会运行——包括升级安装。
;
; 为什么清理逻辑也放在 POSTINSTALL 而不是 POSTUNINSTALL：
; 升级安装时用户可以选择「不先卸载」，NSIS 模板在这种情况下会直接跳到安装流程，
; **根本不执行卸载段**，因此 POSTUNINSTALL 里的清理不会运行，v0.1.0 留下的
; .markdown\ShellNew 会一直残留，「新建」菜单继续显示两条。
; 放在 POSTINSTALL 才能保证升级时一定被清掉。
;
; 键位：与 tauri.conf.json 的 installMode = "currentUser" 对应，写 HKCU。
; 若将来改成 perMachine，这里必须相应改为 HKLM。

!macro NSIS_HOOK_POSTINSTALL
  ; NullFile 表示新建时创建一个 0 字节的空文件——与记事本新建文本文档一致
  WriteRegStr HKCU "Software\Classes\.md\ShellNew" "NullFile" ""

  ; 清理 v0.1.0 的残留：那一版给 .markdown 也注册了 ShellNew，导致「新建」
  ; 菜单出现两条重复项。
  ;
  ; 这里不加「仅升级时执行」的判断：NSIS 模板里的寄存器在此时并不持有可靠的
  ; 旧版安装信息，据此写守卫条件反而可能判断错。无条件删除是安全的——
  ; DeleteRegKey 对不存在的键是无操作；而 v0.1.0 是迄今唯一发布过的版本，
  ; 这个键若存在必然是我们自己写的。何况 .markdown 本就由 plainmark 关联，
  ; 清理它不会影响其它程序的正常关联。
  DeleteRegKey HKCU "Software\Classes\.markdown\ShellNew"

  DetailPrint "已注册 Markdown 文档的新建菜单项"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; 卸载时清理，避免留下孤儿键导致「新建」菜单出现无效项
  DeleteRegKey HKCU "Software\Classes\.md\ShellNew"
  DeleteRegKey HKCU "Software\Classes\.markdown\ShellNew"
!macroend
