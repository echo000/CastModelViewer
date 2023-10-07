// ------------------------------------------------------------------------
// CastModelViewer - Tool to view SEModel Files
// Copyright (C) 2018 Philip/Scobalula
// ------------------------------------------------------------------------
#define TRACE
using System;
using System.Threading;
using System.Collections.Generic;
using System.ComponentModel;
using System.IO;
using System.Diagnostics;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Media.Media3D;
using System.Windows.Media.Imaging;
using Microsoft.Win32;
using System.Linq;
using CastModelViewer;
using CastModelViewer.Util;
using SELib;
using System.Windows.Data;
using System.Windows.Documents;
using System.Reflection;
using System.Windows.Input;
using System.Runtime.InteropServices;
using System.Windows.Interop;
using HelixToolkit.Wpf;
using Cast;

namespace CastModelViewer.Windows
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        #region Properties

        /// <summary>
        /// Number of Models found by Folder Loader
        /// </summary>
        private int ModelsFound = 0;

        /// <summary>
        /// Active Thread
        /// </summary>
        private Thread ActiveThread { get; set; }

        /// <summary>
        /// Watcher Variable for current active thread whether to end or not
        /// </summary>
        private bool EndThread = false;

        /// <summary>
        /// Loaded Models
        /// </summary>
        private List<ModelFile> LoadedModels = new List<ModelFile>();

        /// <summary>
        /// Collection View (For Filtering)
        /// </summary>
        public static CollectionView View { get; set; }

        /// <summary>
        /// Sort Column (for Sorting)
        /// </summary>
        private GridViewColumnHeader ModelListSortColumn = null;

        /// <summary>
        /// Sort Adorner (for Sorting)
        /// </summary>
        private SortAdorner ModelListSortAdorner = null;

        #endregion

        #region MainOperations

        /// <summary>
        /// Main Sausage
        /// </summary>
        public MainWindow()
        {
            Directory.SetCurrentDirectory(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location));
            InitializeComponent();
            ModelList.ItemsSource = LoadedModels;
            View = CollectionViewSource.GetDefaultView(ModelList.ItemsSource) as CollectionView;
            View.Filter = ViewFilter;
            Settings.Load("settings.smvcfg");
            LoadTextures.IsChecked = Settings.Get("loadtexture", "yes") == "yes";
            ShowGrid.IsChecked = Settings.Get("showgrid", "yes") == "yes";
            UseYUpAxis.IsChecked = Settings.Get("yupaxis", "no") == "yes";
            ViewGrid.Visible = Settings.Get("showgrid", "yes") == "yes";
            ModelConverter.RegisterModelConverters();
            Loaded += new RoutedEventHandler(MainWindow_OnLoad);
        }

        /// <summary>
        /// Loads model if one was loaded via Drag and Drop or Open With
        /// </summary>
        private void MainWindow_OnLoad(object sender, RoutedEventArgs e)
        {
            if (Application.Current.Properties["SEModelInput"] != null)
                ProcessModel(Application.Current.Properties["SEModelInput"].ToString(), true);
        }

        #endregion

        #region Utilities

        /// <summary>
        /// Loads SEModels from a Folder and all subfolders.
        /// </summary>
        private List<ModelFile> LoadDirectory(string folder)
        {
            var dirInfo = new DirectoryInfo(folder);

            List<ModelFile> models = new List<ModelFile>();

            if(dirInfo.Name.Contains("models"))
            {
                var result = MessageBox.Show(
                    "CastModelViewer has noticed the folder you dropped on is contains the term \"models\".\n\nIf this folder is from Wraith, Greyhound, or Legion, CastModelViewer can skip scanning the sub-directories and assume each model holds the same name as the folder with its LOD indices to speed up load time.\n\nDo you want CastModelViewer to do that?", "CastModelViewer", MessageBoxButton.YesNoCancel, MessageBoxImage.Information);

                if (result == MessageBoxResult.Yes)
                {
                    var dirs = dirInfo.GetDirectories();

                    foreach (var dir in dirs)
                    {
                        if (EndThread)
                            return models;

                        var model = new ModelFile()
                        {
                            Path = Path.Combine(dir.FullName, dir.Name + "_LOD0.semodel"),
                            BoneCount = 0,
                        };

                        if (!LoadedModels.Contains(model))
                            models.Add(model);
                    }
                }
                else if(result == MessageBoxResult.No)
                {
                    models.AddRange(LoadDirectories(folder));
                }
            }
            else
            {
                models.AddRange(LoadDirectories(folder));
            }

            ModelsFound = 0;
            return models;
        }

        /// <summary>
        /// Loads SEModels from a Folder and all subfolders.
        /// </summary>
        private List<ModelFile> LoadDirectories(string folder)
        {
            List<ModelFile> models = new List<ModelFile>();

            IEnumerable<string> files = Directory.EnumerateFiles(folder);

            foreach (string file in files)
            {
                if (EndThread)
                    break;


                if (file.EndsWith(".semodel") || file.EndsWith(".cast"))
                {
                    var model = new ModelFile()
                    {
                        Path = file,
                        BoneCount = 0,
                    };

                    if (!LoadedModels.Contains(model))
                    {
                        models.Add(model);
                        ModelsFound++;
                        Dispatcher.BeginInvoke(new Action(() => TaskLabel.Content = String.Format("Found {0} models....", ModelsFound)));
                    }
                }
            }


            IEnumerable<string> directories = Directory.EnumerateDirectories(folder);

            foreach (string directory in directories)
            {
                // Skip Texture Folders
                if (directory.EndsWith("_images"))
                    continue;

                if (EndThread)
                    break;

                models.AddRange(LoadDirectories(directory));
            }

            return models;
        }

        /// <summary>
        /// Adds a model to the UI List
        /// </summary>
        private void AddModel(ModelFile model, bool drawModel)
        {
            if (!LoadedModels.Contains(model))
            {
                LoadedModels.Add(model);
            }

            if (drawModel)
            {
                ModelList.SelectedItem = model;
                ModelList.ScrollIntoView(model);
                DrawModel(model);
            }
        }

        /// <summary>
        /// Takes a SEModel Path and adds it to file list
        /// </summary>
        private void ProcessModel(string filePath, bool drawModel = false)
        {
            ModelFile model = new ModelFile()
            {
                Path = filePath,
                BoneCount = 0,
            };

            AddModel(model, drawModel);
        }

        /// <summary>
        /// Loads an SEModel File, if it fails the model is not displayed.
        /// </summary>
        private void DrawModel(ModelFile modelFile)
        {
            if (modelFile == null)
                return;

            try
            {
                var watch = Stopwatch.StartNew();

                //Cast
                if (Path.GetExtension(modelFile.Path) == ".cast")
                {
                    CastModelImporter reader = new CastModelImporter
                    {
                        LoadTextures = LoadTextures.IsChecked == true,
                        UpAxis = UseYUpAxis.IsChecked == true ? "Y" : "Z",
                        Folder = Path.GetDirectoryName(modelFile.Path)
                    };

                    using (FileStream stream = new FileStream(modelFile.Path, FileMode.Open))
                    {
                        Model3DGroup maybemodel = reader.Read(stream);
                        modelFile.ModelBones = reader.ModelBones;
                        modelFile.BoneCount = (int)reader.BoneCount;
                        model.Content = maybemodel;
                        MainViewport.ZoomExtents(0);
                    }

                    watch.Stop();
                    Status.Content = string.Format("Status     : Loaded {0} in {1} Seconds",
                        modelFile.Name, watch.ElapsedMilliseconds / 1000.0);
                    VertexCount.Content = string.Format("Vertices   : {0}", reader.VertexCount);
                    FaceCount.Content = string.Format("Faces      : {0}", reader.FaceCount);
                    MaterialCount.Content = string.Format("Materials  : {0}", reader.MaterialCount);
                    BoneCount.Content = string.Format("Bones      : {0}", reader.BoneCount);
                    Status.Foreground = new SolidColorBrush((Color)ColorConverter.ConvertFromString("#FFABB6C3"));
                    watch.Stop();
                }

                else if (Path.GetExtension(modelFile.Path) == ".semodel")
                {
                    //SEModel
                    SEModelImporter reader = new SEModelImporter
                    {
                        LoadTextures = LoadTextures.IsChecked == true,
                        UpAxis = UseYUpAxis.IsChecked == true ? "Y" : "Z",
                        Folder = Path.GetDirectoryName(modelFile.Path)
                    };

                    using (FileStream stream = new FileStream(modelFile.Path, FileMode.Open))
                    {
                        Model3DGroup loadedModel = reader.Read(stream);
                        modelFile.Materials = reader.SEModelMaterials;
                        modelFile.ModelBones = reader.ModelBones;
                        modelFile.BoneCount = (int)reader.BoneCount;
                        model.Content = loadedModel;
                        MainViewport.ZoomExtents(0);
                    }

                    watch.Stop();
                    Status.Content = string.Format("Status     : Loaded {0} in {1} Seconds",
                        modelFile.Name, watch.ElapsedMilliseconds / 1000.0);
                    VertexCount.Content = string.Format("Vertices   : {0}", reader.VertexCount);
                    FaceCount.Content = string.Format("Faces      : {0}", reader.FaceCount);
                    MaterialCount.Content = string.Format("Materials  : {0}", reader.MaterialCount);
                    BoneCount.Content = string.Format("Bones      : {0}", reader.BoneCount);
                    Status.Foreground = new SolidColorBrush((Color)ColorConverter.ConvertFromString("#FFABB6C3"));
                }
            }
            catch (Exception e)
            {
                Trace.WriteLine(e);
                Status.Content        = string.Format("Status     : Error - {0}", e.Message.Replace("\\", "\\\\"));
                VertexCount.Content   = string.Format("Vertices   : 0");
                FaceCount.Content     = string.Format("Faces      : 0");
                MaterialCount.Content = string.Format("Materials  : 0");
                BoneCount.Content     = string.Format("Bones      : 0");
                Status.Foreground = new SolidColorBrush(Colors.Red);
                return;
            }
        }

        /// <summary>
        /// Loads Bone Counts of all loaded SEModels
        /// </summary>
        private void LoadBoneCounts(List<ModelFile> modelFiles)
        {
            byte[] buffer = new byte[18];

            int i = 0;

            foreach (var modelFile in modelFiles)
            {
                if (EndThread)
                    break;
                UpdateProgress(((double)i / modelFiles.Count) * 100);

                try
                {
                    // Only load if the previous bone count has not been loaded
                    if (!modelFile.BoneCountLoaded)
                    {
                        using (var stream = new FileStream(modelFile.Path, FileMode.Open)) stream.Read(buffer, 0, buffer.Length);
                        modelFile.BoneCount = BitConverter.ToInt32(buffer, 14);
                        modelFile.BoneCountLoaded = true;
                    }
                }
                catch (Exception e)
                {
                    Trace.WriteLine(e);
                    modelFile.BoneCount = -1;
                    modelFile.BoneCountLoaded = false;
                }

                i++;
            }
        }

        /// <summary>
        /// Loads Bone Count from the header of a SEModel file
        /// </summary>
        private void LoadBoneCount(ModelFile modelFile)
        {
            byte[] buffer = new byte[18];

            try
            {
                // Only load if the previous bone count has not been loaded
                if (!modelFile.BoneCountLoaded)
                {
                    using (var stream = new FileStream(modelFile.Path, FileMode.Open)) stream.Read(buffer, 0, buffer.Length);
                    modelFile.BoneCount = BitConverter.ToInt32(buffer, 14);
                    modelFile.BoneCountLoaded = true;
                }
            }
            catch (Exception e)
            {
                Trace.WriteLine(e);
                modelFile.BoneCount = -1;
                modelFile.BoneCountLoaded = false;
            }
        }


        /// <summary>
        /// Recursively builds a tree from bones
        /// </summary>
        /// <param name="bones">Bone list</param>
        /// <param name="bone">Bone</param>
        /// <param name="parent">Parent Tree Item</param>
        private static void BuildBoneTree(
            IEnumerable<ModelFile.ModelBone> bones,
            ModelFile.ModelBone bone, 
            TreeViewItem parent)
        {
            parent.Items.Add(new TreeViewItem() { Header = CustomizeTreeViewItem(bone.Name, "bone") });

            var childBones = bones.Where(i => i.Parent == bone.Index);

            foreach (var childBone in childBones)
                BuildBoneTree(bones, childBone, (TreeViewItem)parent.Items[parent.Items.Count - 1]);
        }

        /// <summary>
        /// Expands a tree view item and all child items
        /// </summary>
        /// <param name="treeItem">Root Item</param>
        private void ExpandAllNodes(TreeViewItem treeItem)
        {
            // https://stackoverflow.com/a/22846911
            treeItem.IsExpanded = true;

            foreach (var childItem in treeItem.Items.OfType<TreeViewItem>())
                ExpandAllNodes(childItem);
        }

        /// <summary>
        /// Customizes a TreeView with an icon and label
        /// </summary>
        /// <param name="itemObj"></param>
        /// <returns></returns>
        private static StackPanel CustomizeTreeViewItem(object itemObj, string imageName)
        {
            // https://stackoverflow.com/a/28718751
            // Add Icon
            // Create Stack Panel
            StackPanel stkPanel = new StackPanel
            {
                Orientation = Orientation.Horizontal
            };

            // Create Image
            stkPanel.Children.Add(new Image
            {
                Source = new BitmapImage(new Uri(String.Format("pack://application:,,,/Icons/{0}.png", imageName))),
                Width = 16,
                Height = 16
            });

            // Create TextBlock
            stkPanel.Children.Add(new TextBlock
            {
                Text = itemObj.ToString()
            });

            return stkPanel;
        }

        /// <summary>
        /// Clears loaded models
        /// </summary>
        private void ClearLoadedModels()
        {
            AbortTaskClick(null, null);
            LoadedModels?.Clear();
            ClearDrawnModel();
            RefreshModelList();
        }

        /// <summary>
        /// Clears the currently drawn model
        /// </summary>
        private void ClearDrawnModel()
        {
            model.Content         = null;
            Status.Content        = "Status     : Idle";
            VertexCount.Content   = "Vertices   : 0";
            FaceCount.Content     = "Faces      : 0";
            MaterialCount.Content = "Materials  : 0";
            BoneCount.Content     = "Bones      : 0";
            Status.Foreground     = new SolidColorBrush((Color)ColorConverter.ConvertFromString("#FFABB6C3"));
        }

        /// <summary>
        /// Resets Progress Bars
        /// </summary>
        private void ResetProgress()
        {
            Dispatcher.BeginInvoke(
                new Action(() =>
                {
                    TaskProgress.Value = 0;
                    TaskBarProgress.ProgressState = System.Windows.Shell.TaskbarItemProgressState.None;
                    TaskBarProgress.ProgressValue = 0;
                    TaskProgress.IsIndeterminate = false;
                    TaskLabel.Content = "Idle";
                }));
        }

        /// <summary>
        /// Updates Progress Bars
        /// </summary>
        /// <param name="value"></param>
        private void UpdateProgress(double value)
        {
            Dispatcher.BeginInvoke(
                new Action(() =>
                {
                    TaskProgress.Value = value;
                    TaskBarProgress.ProgressValue = value / 100.0;
                }));
        }

        /// <summary>
        /// Processes Folder
        /// </summary>
        private void LoadFolder(string path)
        {
            TaskBarProgress.ProgressState = System.Windows.Shell.TaskbarItemProgressState.Normal;
            ActiveThread = new Thread(() =>
            {
                LoadedModels.AddRange(LoadDirectory(path));
                RefreshModelList();
                Dispatcher.BeginInvoke(new Action(() => TaskLabel.Content = "Loading Bone Counts..."));
                LoadBoneCounts(LoadedModels);
                Dispatcher.BeginInvoke(new Action(() => TaskLabel.Content = "Idle"));
                ResetProgress();
            });
            ActiveThread.Start();
        }

        #endregion

        #region CheckBoxOperations

        /// <summary>
        /// Enables/Disables Load Textures setting
        /// </summary>
        private void LoadTextures_Checked(object sender, RoutedEventArgs e)
        {
            Settings.Set("loadtexture", LoadTextures.IsChecked == true ? "yes" : "no");
        }

        /// <summary>
        /// Enables/Disables Grid on Check
        /// </summary>
        private void ShowGrid_Checked(object sender, RoutedEventArgs e)
        {
            Settings.Set("showgrid", ShowGrid.IsChecked == true ? "yes" : "no");
            ViewGrid.Visible = ShowGrid.IsChecked == true;
        }

        /// <summary>
        /// Enables/Disables Grid on Check
        /// </summary>
        private void YUpAxis_Checked(object sender, RoutedEventArgs e)
        {
            Settings.Set("yupaxis", UseYUpAxis.IsChecked == true ? "yes" : "no");
        }

        #endregion

        #region ButtonClickOperations

        /// <summary>
        /// Opens the Wiki
        /// </summary>
        private void Wiki_Click(object sender, RoutedEventArgs e)
        {
            Process.Start("https://github.com/Scobalula/CastModelViewer/wiki");
        }

        /// <summary>
        /// Closes window via File menu exit button
        /// </summary>
        private void AppExit_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }

        /// <summary>
        /// Loads folder of OBJ Files
        /// </summary>
        private void LoadFolderButton_Click(object sender, RoutedEventArgs e)
        {
            if (ActiveThread?.IsAlive == true)
            {
                MessageBox.Show("Please wait for CastModelViewer to finish the current task.", "ERROR", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            var folderDialog = new System.Windows.Forms.FolderBrowserDialog()
            {
                ShowNewFolderButton = false,
                Description = "Select Folder to Import"
            };

            if (folderDialog.ShowDialog() == System.Windows.Forms.DialogResult.OK &&
                !string.IsNullOrWhiteSpace(folderDialog.SelectedPath) &&
                Directory.Exists(folderDialog.SelectedPath))
            {
                TaskLabel.Content = "Loading models...";
                EndThread = false;
                LoadFolder(folderDialog.SelectedPath);
            }
        }

        /// <summary>
        /// Opens and populates Model Info Window
        /// </summary>
        private void ModeInfoClickButton_Click(object sender, RoutedEventArgs e)
        {
            if (ModelList.SelectedItems.Count == 0)
            {
                MessageBox.Show("Please select/load a model first.", "ERROR", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            if (ModelList.SelectedItem is ModelFile modelFile)
            {
                ModelInfoWindow modelInfoWindow = new ModelInfoWindow
                {
                    Owner = this
                };

                // Add a root bone as our starting point
                var rootBone = new ModelFile.ModelBone() { Name = "Root", Index = -1, Parent = 0 };

                modelInfoWindow.InfoList.Items.Add(new TreeViewItem() { Header = String.Format("Bones: {0}", modelFile.ModelBones?.Count) });

                if (modelFile.ModelBones != null)
                    // Recursively build bones, to ensure child bones are parented
                    BuildBoneTree(modelFile.ModelBones, rootBone, (TreeViewItem)modelInfoWindow.InfoList.Items[0]);

                modelInfoWindow.InfoList.Items.Add(new TreeViewItem() { Header = String.Format("Materials: {0}", modelFile.Materials?.Count) });

                var materialItem = (TreeViewItem)modelInfoWindow.InfoList.Items[1];

                // Add Materials and Images
                if (modelFile.Materials != null)
                {
                    foreach (var material in modelFile.Materials)
                    {
                        var materialInfo = material.MaterialData as SEModelSimpleMaterial;

                        var item = new TreeViewItem() { Header = CustomizeTreeViewItem(material.Name, "material") };
                        item.Items.Add(new TreeViewItem() { Header = CustomizeTreeViewItem("Diffuse Map  - " + materialInfo.DiffuseMap, "image") });
                        item.Items.Add(new TreeViewItem() { Header = CustomizeTreeViewItem("Normal Map   - " + materialInfo.NormalMap, "image") });
                        item.Items.Add(new TreeViewItem() { Header = CustomizeTreeViewItem("Specular Map - " + materialInfo.SpecularMap, "image") });

                        materialItem.Items.Add(item);
                    }
                }

                // Gots to expand them views
                ExpandAllNodes((TreeViewItem)modelInfoWindow.InfoList.Items[0]);
                ExpandAllNodes((TreeViewItem)modelInfoWindow.InfoList.Items[1]);

                modelInfoWindow.ShowDialog();
            }
        }

        /// <summary>
        /// Copies model name to clipboard
        /// </summary>
        private void CopyAssetName_Click(object sender, RoutedEventArgs e)
        {
            if (ModelList.SelectedItem is ModelFile modelFile)
                Clipboard.SetText(modelFile?.Name);
        }

        /// <summary>
        /// Copies model file to clipboard
        /// </summary>
        private void CopyAssetFile_Click(object sender, RoutedEventArgs e)
        {
            if (ModelList.SelectedItem is ModelFile modelFile)
                // Have to set this as an array apparently, otherwise Windows won't paste
                Clipboard.SetDataObject(new DataObject(DataFormats.FileDrop, new string[] { modelFile.Path }));
        }

        /// <summary>
        /// Loads SEModels via Dialog
        /// </summary>
        private void LoadButton_Click(object sender, RoutedEventArgs e)
        {
            if (ActiveThread?.IsAlive == true)
            {
                MessageBox.Show("Please wait for CastModelViewer to finish the current task.", "ERROR", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            var openFileDialog = new OpenFileDialog()
            {
                Filter = "Model Files (*.semodel, *.cast)|*.semodel;*.cast",
                Multiselect = true,
                Title = "Select SEModel Files"
            };

            EndThread = false;

            if (openFileDialog.ShowDialog() == true)
            {
                foreach(var file in openFileDialog.FileNames)
                {
                    var model = new ModelFile()
                    {
                        Path = file,
                        BoneCount = 0,
                    };

                    if (!LoadedModels.Contains(model))
                    {
                        LoadedModels.Add(model);
                        LoadBoneCount(model);
                    }
                }
            }

            RefreshModelList();
        }

        /// <summary>
        /// Aborts current task
        /// </summary>
        private void AbortTaskClick(object sender, RoutedEventArgs e)
        {
            TaskProgress.IsIndeterminate = false;
            TaskBarProgress.ProgressState = System.Windows.Shell.TaskbarItemProgressState.None;
            TaskProgress.Value = 0;
            TaskLabel.Content = "Idle";
            EndThread = true;
            ActiveThread?.Join(1500);
        }

        /// <summary>
        /// Clears loaded models on request
        /// </summary>
        private void ClearModels_Click(object sender, RoutedEventArgs e)
        {
            ClearLoadedModels();
        }

        /// <summary>
        /// Opens About Window
        /// </summary>
        private void AboutButton_Click(object sender, RoutedEventArgs e)
        {
            new AboutWindow()
            {
                Owner = this
            }.ShowDialog();
        }

        #endregion

        #region FileDropOperations

        /// <summary>
        /// Processes dropped model onto viewport
        /// </summary>
        private void MainViewport_Drop(object sender, DragEventArgs e)
        {
            if (ActiveThread?.IsAlive == true)
            {
                MessageBox.Show("Please wait for CastModelViewer to finish the current task.", "ERROR", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            EndThread = false;

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                string[] files = (string[])e.Data.GetData(DataFormats.FileDrop);

                foreach (string file in files)
                {
                    if ((Path.GetExtension(file) == ".semodel" || Path.GetExtension(file) == ".cast") && File.Exists(file))
                    {
                        ProcessModel(file, true);
                        RefreshModelList();
                        break;
                    }
                }
            }
        }

        /// <summary>
        /// Processes dropped models/folders onto list
        /// </summary>
        private void ModelList_Drop(object sender, DragEventArgs e)
        {
            if (ActiveThread?.IsAlive == true)
            {
                MessageBox.Show("Please wait for CastModelViewer to finish the current task.", "ERROR", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            EndThread = false;

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                string[] data = (string[])e.Data.GetData(DataFormats.FileDrop);
                foreach(string path in data)
                {
                    if (EndThread)
                        return;

                    if(File.Exists(path))
                    {
                        if(Path.GetExtension(path) == ".semodel" || Path.GetExtension(path) == ".cast")
                        {
                            ProcessModel(path, true);
                            break;
                        }
                    }
                    else if(Directory.Exists(path))
                    {
                        LoadFolder(path);
                        break;
                    }
                }
            }

            RefreshModelList();
        }

        #endregion

        #region MiscOperations

        /// <summary>
        /// Switches model on index change/click.
        /// </summary>
        private void ModelList_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (ModelList.SelectedItem is ModelFile modelFile)
                DrawModel(modelFile);
        }

        /// <summary>
        /// Shuts down thread on close and stores settings
        /// </summary>
        private void Window_Closing(object sender, CancelEventArgs e)
        {
            TaskProgress.IsIndeterminate = false;
            TaskBarProgress.ProgressState = System.Windows.Shell.TaskbarItemProgressState.None;
            TaskProgress.Value = 0;
            TaskLabel.Content = "Idle";
            Settings.Save("settings.smvcfg");
            EndThread = true;
            ActiveThread?.Join(1500);
        }


        /// <summary>
        /// Updates Filter on Search
        /// </summary>
        private void SearchBox_TextChanged(object sender, TextChangedEventArgs e)
        {
            CollectionViewSource.GetDefaultView(ModelList.ItemsSource)?.Refresh();
        }

        /// <summary>
        /// Filters Models by text in Search Box
        /// </summary>
        public bool ViewFilter(object obj)
        {
            return string.IsNullOrEmpty(SearchBox.Text) ? true : (obj.ToString().IndexOf(SearchBox.Text, StringComparison.OrdinalIgnoreCase) >= 0);
        }

        /// <summary>
        /// Sorts Column on click
        /// </summary>
        private void ColumnClick(object sender, RoutedEventArgs e)
        {
            GridViewColumnHeader column = (sender as GridViewColumnHeader);
            string sortBy = column.Tag.ToString();

            if (ModelListSortColumn != null)
            {
                AdornerLayer.GetAdornerLayer(ModelListSortColumn).Remove(ModelListSortAdorner);
                ModelList.Items.SortDescriptions.Clear();
            }

            ListSortDirection newDir = ListSortDirection.Ascending;

            if (ModelListSortColumn == column && ModelListSortAdorner.Direction == newDir)
                newDir = ListSortDirection.Descending;

            ModelListSortColumn = column;
            ModelListSortAdorner = new SortAdorner(ModelListSortColumn, newDir);
            AdornerLayer.GetAdornerLayer(ModelListSortColumn).Add(ModelListSortAdorner);
            ModelList.Items.SortDescriptions.Add(new SortDescription(sortBy, newDir));
        }

        /// <summary>
        /// Refreshes Model List
        /// </summary>
        public void RefreshModelList()
        {
            Dispatcher.BeginInvoke(new Action(() => CollectionViewSource.GetDefaultView(ModelList.ItemsSource)?.Refresh()));
        }

        public void ClearSearch()
        {
            SearchBox.Text = "";
        }

        #endregion

        private void Window_KeyDown(object sender, KeyEventArgs e)
        {
            // Check for CTRL/CTRL+Shift Modifier first
            if(Keyboard.Modifiers == (ModifierKeys.Control | ModifierKeys.Shift) || Keyboard.Modifiers == ModifierKeys.Control)
            {
                switch(e.Key)
                {
                    case Key.C:
                        {
                            if (ModelList.SelectedItem is ModelFile modelFile)
                            {
                                if (Keyboard.Modifiers == (ModifierKeys.Control | ModifierKeys.Shift))
                                    // Have to set this as an array apparently, otherwise Windows won't paste
                                    Clipboard.SetDataObject(new DataObject(DataFormats.FileDrop, new string[] { modelFile.Path }));
                                // Just copy name
                                else
                                    Clipboard.SetText(modelFile?.Name);
                            }
                            break;
                        }
                    case Key.X:
                        {
                            if (Keyboard.Modifiers == (ModifierKeys.Control | ModifierKeys.Shift))
                                ClearLoadedModels();
                            else
                                AbortTaskClick(null, null);
                            break;
                        }
                }
            }

        }

        private void AbortTaskDoubleClick(object sender, MouseButtonEventArgs e)
        {
            ClearLoadedModels();
        }
    }
}
