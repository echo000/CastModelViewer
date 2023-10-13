// ------------------------------------------------------------------------
// CastModelViewer - Tool to view SEModel Files
// Copyright (C) 2018 Philip/Scobalula
// ------------------------------------------------------------------------
using HelixToolkit.Wpf;
using SELib;
using SELib.Utilities;
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
    /// <summary>
    /// SEModel Helix Importer
    /// Handles loading SEModel Data for the Helix Viewport
    /// </summary>
    class SEModelImporter : ModelReader
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
        /// SEModel Materials
        /// </summary>
        public readonly List<SEModelMaterial> SEModelMaterials = new List<SEModelMaterial>();

        /// <summary>
        /// Helix Materials
        /// </summary>
        private readonly List<Material> Materials = new List<Material>();

        /// <summary>
        /// Axis Values
        /// </summary>
        public Dictionary<string, Vector3[]> Axes = new Dictionary<string, Vector3[]>()
        {
            { "Z", new Vector3[]
            {
                new Vector3(1.000000f, 0.000000f, 0.000000f),
                new Vector3(0.000000f, 1.000000f, 0.000000f),
                new Vector3(0.000000f, 0.000000f, 1.000000f),
            }
            },
            { "Y", new Vector3[]
            {
                new Vector3(1.000000f, 0.000000f, 0.000000f),
                new Vector3(0.000000f, 0.000000f, 1.000000f),
                new Vector3(0.000000f, 1.000000f, 0.000000f),
            }
            },
        };

        /// <summary>
        /// Computes Dot Product of the 2 Vectors
        /// </summary>
        /// <param name="a"></param>
        /// <param name="b"></param>
        /// <returns></returns>
        public float DotProduct(Vector3 a, Vector3 b)
        {
            return (float)((a.X * b.X) + (a.Y * b.Y) + (a.Z * b.Z));
        }

        /// <summary>
        /// Loads SEModel
        /// </summary>
        public override Model3DGroup Read(Stream s)
        {
            SEModel semodel = SEModel.Read(s);

            Model3DGroup modelGroup = new Model3DGroup();

            MaterialCount = semodel.Materials.Count;
            BoneCount = semodel.BoneCount;

            ModelBones = new List<ModelFile.ModelBone>();

            LoadBones(semodel);

            LoadMaterials(semodel);

            foreach(var semesh in semodel.Meshes)
            {
                // Validate material index
                if (semesh.MaterialReferenceIndicies[0] < 0 || semesh.MaterialReferenceIndicies[0] > MaterialCount)
                    continue;

                Mesh mesh = new Mesh
                {
                    Positions = new List<Point3D>(),
                    TriangleIndices = new List<int>(),
                    TextureCoordinates = new List<Point>(),
                    Normals = new List<Vector3D>(),
                    Material = Materials[semesh.MaterialReferenceIndicies[0]]
                };

                VertexCount += semesh.VertexCount;
                FaceCount += semesh.FaceCount;

                foreach(var vertex in semesh.Verticies)
                {
                    var position = new Vector3(
                        DotProduct(vertex.Position, Axes[UpAxis][0]),
                        DotProduct(vertex.Position, Axes[UpAxis][1]),
                        DotProduct(vertex.Position, Axes[UpAxis][2])
                        );
                    var normal = new Vector3(
                        DotProduct(vertex.VertexNormal, Axes[UpAxis][0]),
                        DotProduct(vertex.VertexNormal, Axes[UpAxis][1]),
                        DotProduct(vertex.VertexNormal, Axes[UpAxis][2])
                        );

                    mesh.Positions.Add(new Point3D(position.X, position.Y, position.Z));
                    mesh.TextureCoordinates.Add(new Point(vertex.UVSets[0].X, vertex.UVSets[0].Y));
                    mesh.Normals.Add(new Vector3D(normal.X, normal.Y, normal.Z));
                }

                foreach(var face in semesh.Faces )
                {
                    mesh.TriangleIndices.Add((int)face.FaceIndex1);
                    mesh.TriangleIndices.Add((int)face.FaceIndex2);
                    mesh.TriangleIndices.Add((int)face.FaceIndex3);
                }

                modelGroup.Children.Add(mesh.CreateModel());
            }

            return modelGroup;
        }

        /// <summary>
        /// Loads Bone Names and Offsets (As a string formatted)
        /// </summary>
        private void LoadBones(SEModel semodel)
        {
            for(int i = 0; i < semodel.Bones.Count; i++)
            {
                ModelBones.Add(new ModelFile.ModelBone()
                {
                    Name     = semodel.Bones[i].BoneName,
                    Index    = i,
                    Parent   = semodel.Bones[i].BoneParent,
                    Position = semodel.Bones[i].GlobalPosition
                });
            }
        }

        /// <summary>
        /// Loads materials and textures (if they exist)
        /// </summary>
        private void LoadMaterials(SEModel semodel)
        {
            foreach(var material in semodel.Materials)
            {
                SEModelMaterials.Add(material);
                var materialGroup = new MaterialGroup();


                var data = material.MaterialData as SEModelSimpleMaterial;
                string image = Path.Combine(Folder, data.DiffuseMap);

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
            public Material Material { get; set; }

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
