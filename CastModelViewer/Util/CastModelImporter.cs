using Cast;
using HelixToolkit.Wpf;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Media.Media3D;

namespace CastModelViewer.Util
{
    internal class CastModelImporter : ModelReader
    {
        /// <summary>
        /// Accepted Image Formats for Textures
        /// </summary>
        public static string[] AcceptedImageExtensions =
        {
            ".PNG",
            ".TIF",
            ".TIFF",
            ".JPG",
            ".JPEG",
            ".BMP",
        };

        /// <summary>
        /// Number of Materials
        /// </summary>
        public int MaterialCount { get; set; }

        /// <summary>
        /// Number of Vertices
        /// </summary>
        public uint VertexCount { get; set; }

        /// <summary>
        /// Number of Faces
        /// </summary>
        public uint FaceCount { get; set; }

        /// <summary>
        /// Number of Bones
        /// </summary>
        public uint BoneCount { get; set; }

        /// <summary>
        /// Load Textures from Material Data
        /// </summary>
        public bool LoadTextures { get; set; }

        /// <summary>
        /// Folder Path (For material loader)
        /// </summary>
        public string Folder { get; set; }

        /// <summary>
        /// Model Up Axis (X or Y)
        /// </summary>
        public string UpAxis { get; set; }

        /// <summary>
        /// Random Int (For material loader)
        /// </summary>
        private readonly Random RandomInt = new Random();

        /// <summary>
        /// Bones in this Model
        /// </summary>
        public List<ModelFile.ModelBone> ModelBones { get; set; }

        /// <summary>
        /// Helix Materials
        /// </summary>
        private readonly List<System.Windows.Media.Media3D.Material> Materials = new List<System.Windows.Media.Media3D.Material>();

        /// <summary>
        /// Cast stores materials as hashes, this is a list of hashes to match with the materials list
        /// </summary>
        private readonly List<ulong> MaterialHashes = new List<ulong>();

        /// <summary>
        /// Loads the Cast Model
        /// </summary>
        public override Model3DGroup Read(Stream s)
        {
            var cast = CastFile.Load(s);
            Model3DGroup modelGroup = new Model3DGroup();
            ModelBones = new List<ModelFile.ModelBone>();
            var model = cast.RootNodes[0].ChildrenOfType<Cast.Model>().FirstOrDefault();
            var skeleton = model.Skeleton();
            BoneCount = (uint)skeleton.ChildNodes.Count;
            MaterialCount = model.ChildrenOfType<Cast.Material>().Count();

            LoadBones(skeleton);
            LoadMaterials(model);
            var meshes = model.ChildrenOfType<Cast.Mesh>();

            foreach(var cmesh in meshes)
            {
                Mesh mesh = new Mesh
                {
                    Positions = new List<Point3D>(),
                    TriangleIndices = new List<int>(),
                    TextureCoordinates = new List<Point>(),
                    Normals = new List<Vector3D>(),
                    Material = Materials[MaterialHashes.IndexOf((ulong)cmesh.Properties["m"].Values[0])]
                };

                cmesh.Properties.TryGetValue("vp", out var positions);
                for(int i = 0; i < positions.Values.Count; i++)
                {
                    VertexCount++;
                    mesh.Positions.Add(new Point3D(
                        ((Vector3)positions.Values[i]).X,
                        ((Vector3)positions.Values[i]).Y,
                        ((Vector3)positions.Values[i]).Z));
                }

                cmesh.Properties.TryGetValue("vn", out var normals);
                for (int i = 0; i < normals.Values.Count; i++)
                {
                    mesh.Normals.Add(new Vector3D(
                            ((Vector3)normals.Values[i]).X,
                            ((Vector3)normals.Values[i]).Y,
                            ((Vector3)normals.Values[i]).Z));
                }
                cmesh.Properties.TryGetValue("u0", out var uvs);
                for(int i = 0; i < uvs.Values.Count; i++)
                {
                    mesh.TextureCoordinates.Add(
                        new Point(
                                ((Vector2)uvs.Values[i]).X,
                                ((Vector2)uvs.Values[i]).Y));
                }

                cmesh.Properties.TryGetValue("f", out var faces);
                for (var i = 0; i < faces.Values.Count; i += 3)
                {
                    FaceCount++;
                    mesh.TriangleIndices.Add(Convert.ToInt32(faces.Values[i]));
                    mesh.TriangleIndices.Add(Convert.ToInt32(faces.Values[i + 1]));
                    mesh.TriangleIndices.Add(Convert.ToInt32(faces.Values[i + 2]));
                }
                modelGroup.Children.Add(mesh.CreateModel());
            }

            return modelGroup;
        }

        /// <summary>
        /// Loads Bone Names and Offsets (As a string formatted)
        /// </summary>
        private void LoadBones(Skeleton skeleton)
        {
            for (int i = 0; i < skeleton.ChildNodes.Count; i++)
            {
                ModelBones.Add(new ModelFile.ModelBone()
                {
                    Name = (string)skeleton.ChildNodes[i].Properties["n"].Values[0],
                    Index = i,
                    Parent = (int)skeleton.ChildNodes[i].Properties["p"].Values[0],
                    Position = new SELib.Utilities.Vector3(((Vector3)skeleton.ChildNodes[i].Properties["wp"].Values[0]).X,
                    ((Vector3)skeleton.ChildNodes[i].Properties["wp"].Values[0]).Y,
                    ((Vector3)skeleton.ChildNodes[i].Properties["wp"].Values[0]).Z)
                });
            }
        }

        /// <summary>
        /// Loads materials and textures (if they exist)
        /// </summary>
        private void LoadMaterials(Model model)
        {
            var materials = model.ChildrenOfType<Cast.Material>();
            foreach (var material in materials)
            {
                var materialGroup = new MaterialGroup();
                MaterialHashes.Add(material.Hash);
                string image = Path.Combine(Folder, (string)material.ChildNodes[0].Properties["p"].Values[0]);

                // If we have an image, we can load it, otherwise, assign a random color
                if (File.Exists(image) && AcceptedImageExtensions.Contains(Path.GetExtension(image), StringComparer.CurrentCultureIgnoreCase) && LoadTextures == true)
                {
                    materialGroup.Children.Add(new DiffuseMaterial(CreateTextureBrush(image)));
                }
                else
                {
                    materialGroup.Children.Add(new DiffuseMaterial(new SolidColorBrush(
                    System.Windows.Media.Color.FromRgb
                    (
                        (byte)RandomInt.Next(128, 255),
                        (byte)RandomInt.Next(128, 255),
                        (byte)RandomInt.Next(128, 255)
                    ))));
                }
                Materials.Add(materialGroup);
            }
        }

        /// <summary>
        /// Loads texture
        /// </summary>
        private ImageBrush CreateTextureBrush(string path)
        {
            var img = new BitmapImage(new Uri(path, UriKind.Relative));
            var textureBrush = new ImageBrush(img) { Opacity = 1.0, ViewportUnits = BrushMappingMode.Absolute, TileMode = TileMode.Tile };
            return textureBrush;
        }

        /// <summary>
        /// Mesh Data
        /// </summary>
        private class Mesh
        {
            /// <summary>
            /// Vertex Positions
            /// </summary>
            public List<Point3D> Positions { get; set; }

            /// <summary>
            /// Face Indices
            /// </summary>
            public List<int> TriangleIndices { get; set; }

            /// <summary>
            /// UV Positions
            /// </summary>
            public List<Point> TextureCoordinates { get; set; }

            /// <summary>
            /// Vertex Normals
            /// </summary>
            public List<Vector3D> Normals { get; set; }

            /// <summary>
            /// Mesh Material
            /// </summary>
            public System.Windows.Media.Media3D.Material Material { get; set; }

            /// <summary>
            /// Creates a Model from Mesh Data
            /// </summary>
            /// <returns></returns>
            public Model3D CreateModel()
            {
                var geometry = new MeshGeometry3D
                {
                    Positions = new Point3DCollection(Positions),
                    TriangleIndices = new Int32Collection(TriangleIndices),
                    Normals = new Vector3DCollection(Normals)
                };
                if (TextureCoordinates != null)
                {
                    geometry.TextureCoordinates = new PointCollection(TextureCoordinates);
                }

                return new GeometryModel3D(geometry, Material) { BackMaterial = Material };
            }
        }
    }
}
