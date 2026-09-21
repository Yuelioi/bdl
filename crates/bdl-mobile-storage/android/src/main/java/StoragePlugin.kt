package com.yueli.bdl.storage

import android.Manifest
import android.app.Activity
import android.content.ClipboardManager
import android.content.ContentValues
import android.content.Context
import android.content.ActivityNotFoundException
import android.content.Intent
import android.media.MediaScannerConnection
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.util.Base64
import android.webkit.MimeTypeMap
import androidx.activity.result.ActivityResult
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.io.FileInputStream

@InvokeArg
class ExportFileArgs {
  lateinit var sourcePath: String
  lateinit var treeUri: String
  lateinit var relativePath: String
  lateinit var duplicateNamingStrategy: String
}

@InvokeArg
class OpenExportArgs {
  lateinit var treeUri: String
  lateinit var relativePath: String
  var documentUri: String? = null
}

@InvokeArg
class SaveImageArgs {
  lateinit var fileName: String
  lateinit var mimeType: String
  lateinit var base64Data: String
}

@TauriPlugin(
  permissions = [
    Permission(
      strings = [Manifest.permission.WRITE_EXTERNAL_STORAGE],
      alias = "legacyGallery",
    ),
  ],
)
class StoragePlugin(private val activity: Activity) : Plugin(activity) {
  @Command
  fun readClipboardText(invoke: Invoke) {
    try {
      val clipboard = activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
      val clip = clipboard.primaryClip
      val text =
        if (clip != null && clip.itemCount > 0) {
          clip.getItemAt(0).coerceToText(activity)?.toString().orEmpty()
        } else {
          ""
        }
      val response = JSObject()
      response.put("text", text)
      invoke.resolve(response)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "无法读取剪贴板")
    }
  }

  @Command
  fun legacyGalleryPermissionRequired(invoke: Invoke) {
    val response = JSObject()
    response.put("required", Build.VERSION.SDK_INT < Build.VERSION_CODES.Q)
    invoke.resolve(response)
  }

  @Command
  fun saveImageToGallery(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(SaveImageArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "图片保存参数无效")
      return
    }

    Thread {
      try {
        val response = saveImage(args)
        invoke.resolve(response)
      } catch (error: SecurityException) {
        invoke.reject("没有相册写入权限")
      } catch (error: Exception) {
        invoke.reject(error.message ?: "保存图片到相册失败")
      }
    }.start()
  }

  @Command
  fun pickDirectory(invoke: Invoke) {
    val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE)
    intent.addFlags(
      Intent.FLAG_GRANT_READ_URI_PERMISSION or
        Intent.FLAG_GRANT_WRITE_URI_PERMISSION or
        Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION or
        Intent.FLAG_GRANT_PREFIX_URI_PERMISSION
    )
    startActivityForResult(invoke, intent, "pickDirectoryResult")
  }

  @ActivityCallback
  fun pickDirectoryResult(invoke: Invoke, result: ActivityResult) {
    if (result.resultCode == Activity.RESULT_CANCELED) {
      invoke.reject("已取消选择保存目录")
      return
    }
    if (result.resultCode != Activity.RESULT_OK) {
      invoke.reject("选择保存目录失败")
      return
    }

    val data = result.data
    val treeUri = data?.data
    if (treeUri == null) {
      invoke.reject("系统未返回保存目录")
      return
    }

    try {
      val takeFlags =
        data.flags and (Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
      activity.contentResolver.takePersistableUriPermission(treeUri, takeFlags)
      val response = JSObject()
      response.put("tree_uri", treeUri.toString())
      response.put("display_name", displayName(treeUri))
      invoke.resolve(response)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "无法保存目录访问权限")
    }
  }

  @Command
  fun exportFile(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(ExportFileArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "导出参数无效")
      return
    }

    Thread {
      try {
        invoke.resolve(export(args))
      } catch (error: SecurityException) {
        invoke.reject("保存目录权限已失效，请重新选择目录")
      } catch (error: Exception) {
        invoke.reject(error.message ?: "文件导出失败")
      }
    }.start()
  }

  @Command
  fun openExportFile(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(OpenExportArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "打开文件参数无效")
      return
    }

    try {
      val documentUri = resolveExportDocument(args)
      val type = documentMimeType(documentUri) ?: mimeType(relativeParts(args.relativePath).last())
      val intent = Intent(Intent.ACTION_VIEW).apply {
        setDataAndType(documentUri, type)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
      }
      try {
        activity.startActivity(intent)
      } catch (_: ActivityNotFoundException) {
        openDirectoryIntent(resolveExportDirectory(args))
      }
      invoke.resolve(JSObject())
    } catch (error: SecurityException) {
      invoke.reject("保存目录权限已失效，请重新选择目录")
    } catch (error: Exception) {
      invoke.reject(error.message ?: "无法打开导出文件")
    }
  }

  @Command
  fun openExportDirectory(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(OpenExportArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "打开目录参数无效")
      return
    }

    try {
      openDirectoryIntent(resolveExportDirectory(args))
      invoke.resolve(JSObject())
    } catch (error: SecurityException) {
      invoke.reject("保存目录权限已失效，请重新选择目录")
    } catch (error: Exception) {
      invoke.reject(error.message ?: "无法打开导出目录")
    }
  }

  private fun export(args: ExportFileArgs): JSObject {
    val source = File(args.sourcePath)
    require(source.isFile) { "待导出的文件不存在" }

    val parts = args.relativePath
      .replace('\\', '/')
      .split('/')
      .filter { it.isNotBlank() && it != "." }
    require(parts.isNotEmpty() && parts.none { it == ".." }) { "导出相对路径无效" }

    val treeUri = Uri.parse(args.treeUri)
    var parent = treeRoot(treeUri)
    for (segment in parts.dropLast(1)) {
      parent = ensureDirectory(treeUri, parent, segment)
    }

    val requestedName = parts.last()
    val existing = findChild(treeUri, parent, requestedName)
    if (args.duplicateNamingStrategy == "skip_existing" && existing != null) {
      return exportResponse(
        "skipped_existing",
        parts.dropLast(1).plus(requestedName).joinToString("/"),
        existing
      )
    }

    val finalName = when (args.duplicateNamingStrategy) {
      "append_suffix" -> availableName(treeUri, parent, requestedName)
      "overwrite_existing", "skip_existing" -> requestedName
      else -> throw IllegalArgumentException("未知重名处理策略")
    }
    val overwritten =
      if (args.duplicateNamingStrategy == "overwrite_existing") existing else null
    val tempName = ".bdl-${System.nanoTime()}-$finalName"
    val tempUri = DocumentsContract.createDocument(
      activity.contentResolver,
      parent,
      mimeType(finalName),
      tempName
    ) ?: throw IllegalStateException("无法创建临时导出文件")

    var committed = false
    var backupUri: Uri? = null
    try {
      FileInputStream(source).use { input ->
        activity.contentResolver.openOutputStream(tempUri, "w").use { output ->
          requireNotNull(output) { "无法打开目标文件" }
          input.copyTo(output)
          output.flush()
        }
      }

      if (overwritten != null) {
        val backupName = ".bdl-backup-${System.nanoTime()}-$finalName"
        backupUri = DocumentsContract.renameDocument(
          activity.contentResolver,
          overwritten,
          backupName
        ) ?: throw IllegalStateException("无法备份已有文件")
      }

      val finalUri = DocumentsContract.renameDocument(
        activity.contentResolver,
        tempUri,
        finalName
      ) ?: throw IllegalStateException("无法提交导出文件")
      committed = true
      backupUri?.let { backup ->
        runCatching {
          DocumentsContract.deleteDocument(activity.contentResolver, backup)
        }
      }
      return exportResponse(
        "exported",
        parts.dropLast(1).plus(finalName).joinToString("/"),
        finalUri
      )
    } finally {
      if (!committed) {
        runCatching {
          DocumentsContract.deleteDocument(activity.contentResolver, tempUri)
        }
        backupUri?.let { backup ->
          runCatching {
            if (findChild(treeUri, parent, finalName) == null) {
              DocumentsContract.renameDocument(activity.contentResolver, backup, finalName)
            }
          }
        }
      }
    }
  }

  private fun saveImage(args: SaveImageArgs): JSObject {
    val fileName = File(args.fileName).name
    require(fileName.isNotBlank()) { "图片文件名无效" }
    require(args.mimeType.startsWith("image/")) { "图片类型无效" }
    val bytes = Base64.decode(args.base64Data, Base64.DEFAULT)
    require(bytes.isNotEmpty()) { "图片内容为空" }

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
      val resolver = activity.contentResolver
      val values = ContentValues().apply {
        put(MediaStore.Images.Media.DISPLAY_NAME, fileName)
        put(MediaStore.Images.Media.MIME_TYPE, args.mimeType)
        put(MediaStore.Images.Media.RELATIVE_PATH, "${Environment.DIRECTORY_PICTURES}/BDL")
        put(MediaStore.Images.Media.IS_PENDING, 1)
      }
      val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values)
        ?: throw IllegalStateException("无法创建相册图片")
      var committed = false
      try {
        resolver.openOutputStream(uri, "w").use { output ->
          requireNotNull(output) { "无法写入相册图片" }
          output.write(bytes)
          output.flush()
        }
        values.clear()
        values.put(MediaStore.Images.Media.IS_PENDING, 0)
        resolver.update(uri, values, null, null)
        committed = true
        return galleryResponse(uri.toString())
      } finally {
        if (!committed) {
          runCatching { resolver.delete(uri, null, null) }
        }
      }
    }

    val pictures = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES)
    val directory = File(pictures, "BDL")
    if (!directory.exists() && !directory.mkdirs()) {
      throw IllegalStateException("无法创建 BDL 相册目录")
    }
    val output = uniqueFile(directory, fileName)
    output.writeBytes(bytes)
    MediaScannerConnection.scanFile(
      activity,
      arrayOf(output.absolutePath),
      arrayOf(args.mimeType),
      null,
    )
    return galleryResponse(output.absolutePath)
  }

  private fun uniqueFile(directory: File, requestedName: String): File {
    val direct = File(directory, requestedName)
    if (!direct.exists()) return direct
    val dot = requestedName.lastIndexOf('.')
    val hasExtension = dot > 0 && dot < requestedName.length - 1
    val stem = if (hasExtension) requestedName.substring(0, dot) else requestedName
    val extension = if (hasExtension) requestedName.substring(dot) else ""
    var suffix = 1
    while (suffix < 10000) {
      val candidate = File(directory, "$stem ($suffix)$extension")
      if (!candidate.exists()) return candidate
      suffix += 1
    }
    throw IllegalStateException("无法生成可用的图片文件名")
  }

  private fun galleryResponse(uri: String): JSObject {
    val response = JSObject()
    response.put("uri", uri)
    return response
  }

  private fun resolveExportDocument(args: OpenExportArgs): Uri {
    args.documentUri?.takeIf { it.isNotBlank() }?.let { return Uri.parse(it) }
    val treeUri = Uri.parse(args.treeUri)
    val parts = relativeParts(args.relativePath)
    var parent = treeRoot(treeUri)
    for (segment in parts.dropLast(1)) {
      parent = findChild(treeUri, parent, segment)
        ?: throw IllegalStateException("找不到导出目录：$segment")
      if (documentMimeType(parent) != DocumentsContract.Document.MIME_TYPE_DIR) {
        throw IllegalStateException("导出路径中的目录已不存在：$segment")
      }
    }
    return findChild(treeUri, parent, parts.last())
      ?: throw IllegalStateException("导出文件已不存在")
  }

  private fun resolveExportDirectory(args: OpenExportArgs): Uri {
    val treeUri = Uri.parse(args.treeUri)
    val parts = relativeParts(args.relativePath)
    var parent = treeRoot(treeUri)
    for (segment in parts.dropLast(1)) {
      parent = findChild(treeUri, parent, segment)
        ?: throw IllegalStateException("找不到导出目录：$segment")
      if (documentMimeType(parent) != DocumentsContract.Document.MIME_TYPE_DIR) {
        throw IllegalStateException("导出路径中的目录已不存在：$segment")
      }
    }
    return parent
  }

  private fun openDirectoryIntent(directoryUri: Uri) {
    val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
    val viewIntent = Intent(Intent.ACTION_VIEW).apply {
      setDataAndType(directoryUri, DocumentsContract.Document.MIME_TYPE_DIR)
      addFlags(flags)
    }
    try {
      activity.startActivity(viewIntent)
      return
    } catch (_: ActivityNotFoundException) {
      // Some Android file managers do not register ACTION_VIEW for document-tree directories.
    }

    val pickerIntent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
      addFlags(
        flags or
          Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION or
          Intent.FLAG_GRANT_PREFIX_URI_PERMISSION
      )
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
        putExtra(DocumentsContract.EXTRA_INITIAL_URI, directoryUri)
      }
    }
    activity.startActivity(pickerIntent)
  }

  private fun relativeParts(relativePath: String): List<String> {
    val parts = relativePath
      .replace('\\', '/')
      .split('/')
      .filter { it.isNotBlank() && it != "." }
    require(parts.isNotEmpty() && parts.none { it == ".." }) { "导出相对路径无效" }
    return parts
  }

  private fun treeRoot(treeUri: Uri): Uri {
    val rootId = DocumentsContract.getTreeDocumentId(treeUri)
    return DocumentsContract.buildDocumentUriUsingTree(treeUri, rootId)
  }

  private fun ensureDirectory(treeUri: Uri, parent: Uri, name: String): Uri {
    val child = findChild(treeUri, parent, name)
    if (child != null) {
      if (documentMimeType(child) != DocumentsContract.Document.MIME_TYPE_DIR) {
        throw IllegalStateException("导出路径中的目录名已被同名文件占用：$name")
      }
      return child
    }
    return DocumentsContract.createDocument(
      activity.contentResolver,
      parent,
      DocumentsContract.Document.MIME_TYPE_DIR,
      name
    ) ?: throw IllegalStateException("无法创建导出目录：$name")
  }

  private fun findChild(treeUri: Uri, parent: Uri, name: String): Uri? {
    val parentId = DocumentsContract.getDocumentId(parent)
    val children = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, parentId)
    val columns = arrayOf(
      DocumentsContract.Document.COLUMN_DOCUMENT_ID,
      DocumentsContract.Document.COLUMN_DISPLAY_NAME
    )
    activity.contentResolver.query(children, columns, null, null, null).use { cursor ->
      if (cursor == null) return null
      val idIndex = cursor.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
      val nameIndex = cursor.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
      while (cursor.moveToNext()) {
        if (cursor.getString(nameIndex) == name) {
          return DocumentsContract.buildDocumentUriUsingTree(treeUri, cursor.getString(idIndex))
        }
      }
    }
    return null
  }

  private fun availableName(treeUri: Uri, parent: Uri, requestedName: String): String {
    if (findChild(treeUri, parent, requestedName) == null) return requestedName
    val dot = requestedName.lastIndexOf('.')
    val hasExtension = dot > 0 && dot < requestedName.length - 1
    val stem = if (hasExtension) requestedName.substring(0, dot) else requestedName
    val extension = if (hasExtension) requestedName.substring(dot) else ""
    var suffix = 1
    while (suffix < 10000) {
      val candidate = "$stem ($suffix)$extension"
      if (findChild(treeUri, parent, candidate) == null) return candidate
      suffix += 1
    }
    throw IllegalStateException("无法生成可用的重名文件名")
  }

  private fun displayName(uri: Uri): String {
    val document = treeRoot(uri)
    val column = DocumentsContract.Document.COLUMN_DISPLAY_NAME
    activity.contentResolver.query(document, arrayOf(column), null, null, null).use { cursor ->
      if (cursor != null && cursor.moveToFirst()) {
        return cursor.getString(cursor.getColumnIndexOrThrow(column))
      }
    }
    return uri.lastPathSegment ?: "已选择目录"
  }

  private fun documentMimeType(uri: Uri): String? {
    val column = DocumentsContract.Document.COLUMN_MIME_TYPE
    activity.contentResolver.query(uri, arrayOf(column), null, null, null).use { cursor ->
      if (cursor != null && cursor.moveToFirst()) {
        return cursor.getString(cursor.getColumnIndexOrThrow(column))
      }
    }
    return null
  }

  private fun mimeType(fileName: String): String {
    val extension = fileName.substringAfterLast('.', "").lowercase()
    return MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension)
      ?: "application/octet-stream"
  }

  private fun exportResponse(outcome: String, relativePath: String, uri: Uri): JSObject {
    val response = JSObject()
    response.put("outcome", outcome)
    response.put("relative_path", relativePath)
    response.put("document_uri", uri.toString())
    return response
  }
}
