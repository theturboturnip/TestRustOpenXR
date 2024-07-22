; SPIR-V
; Version: 1.3
; Generator: Google Shaderc over Glslang; 11
; Bound: 64
; Schema: 0
               OpCapability Shader
               OpCapability MultiView
          %1 = OpExtInstImport "GLSL.std.450"
               OpMemoryModel Logical GLSL450
               OpEntryPoint Vertex %main "main" %_ %gl_ViewIndex %gl_VertexIndex %xy
               OpSource GLSL 450
               OpSourceExtension "GL_EXT_multiview"
               OpSourceExtension "GL_GOOGLE_cpp_style_line_directive"
               OpSourceExtension "GL_GOOGLE_include_directive"
               OpName %main "main"
               OpName %vertices "vertices"
               OpName %gl_PerVertex "gl_PerVertex"
               OpMemberName %gl_PerVertex 0 "gl_Position"
               OpMemberName %gl_PerVertex 1 "gl_PointSize"
               OpMemberName %gl_PerVertex 2 "gl_ClipDistance"
               OpMemberName %gl_PerVertex 3 "gl_CullDistance"
               OpName %_ ""
               OpName %matrices "matrices"
               OpMemberName %matrices 0 "eye_screen_from_world"
               OpName %__0 ""
               OpName %gl_ViewIndex "gl_ViewIndex"
               OpName %gl_VertexIndex "gl_VertexIndex"
               OpName %xy "xy"
               OpMemberDecorate %gl_PerVertex 0 BuiltIn Position
               OpMemberDecorate %gl_PerVertex 1 BuiltIn PointSize
               OpMemberDecorate %gl_PerVertex 2 BuiltIn ClipDistance
               OpMemberDecorate %gl_PerVertex 3 BuiltIn CullDistance
               OpDecorate %gl_PerVertex Block
               OpDecorate %_arr_mat4v4float_uint_2 ArrayStride 64
               OpMemberDecorate %matrices 0 ColMajor
               OpMemberDecorate %matrices 0 Offset 0
               OpMemberDecorate %matrices 0 MatrixStride 16
               OpDecorate %matrices Block
               OpDecorate %__0 DescriptorSet 0
               OpDecorate %__0 Binding 0
               OpDecorate %gl_ViewIndex BuiltIn ViewIndex
               OpDecorate %gl_VertexIndex BuiltIn VertexIndex
               OpDecorate %xy Location 0
       %void = OpTypeVoid
          %3 = OpTypeFunction %void
      %float = OpTypeFloat 32
    %v2float = OpTypeVector %float 2
       %uint = OpTypeInt 32 0
     %uint_6 = OpConstant %uint 6
%_arr_v2float_uint_6 = OpTypeArray %v2float %uint_6
%_ptr_Function__arr_v2float_uint_6 = OpTypePointer Function %_arr_v2float_uint_6
   %float_n1 = OpConstant %float -1
         %14 = OpConstantComposite %v2float %float_n1 %float_n1
    %float_1 = OpConstant %float 1
         %16 = OpConstantComposite %v2float %float_1 %float_n1
         %17 = OpConstantComposite %v2float %float_1 %float_1
         %18 = OpConstantComposite %v2float %float_n1 %float_1
         %19 = OpConstantComposite %_arr_v2float_uint_6 %14 %16 %17 %14 %17 %18
    %v4float = OpTypeVector %float 4
     %uint_1 = OpConstant %uint 1
%_arr_float_uint_1 = OpTypeArray %float %uint_1
%gl_PerVertex = OpTypeStruct %v4float %float %_arr_float_uint_1 %_arr_float_uint_1
%_ptr_Output_gl_PerVertex = OpTypePointer Output %gl_PerVertex
          %_ = OpVariable %_ptr_Output_gl_PerVertex Output
        %int = OpTypeInt 32 1
      %int_0 = OpConstant %int 0
%mat4v4float = OpTypeMatrix %v4float 4
     %uint_2 = OpConstant %uint 2
%_arr_mat4v4float_uint_2 = OpTypeArray %mat4v4float %uint_2
   %matrices = OpTypeStruct %_arr_mat4v4float_uint_2
%_ptr_Uniform_matrices = OpTypePointer Uniform %matrices
        %__0 = OpVariable %_ptr_Uniform_matrices Uniform
%_ptr_Input_int = OpTypePointer Input %int
%gl_ViewIndex = OpVariable %_ptr_Input_int Input
%_ptr_Uniform_mat4v4float = OpTypePointer Uniform %mat4v4float
%gl_VertexIndex = OpVariable %_ptr_Input_int Input
%_ptr_Function_v2float = OpTypePointer Function %v2float
  %float_0_5 = OpConstant %float 0.5
   %float_n2 = OpConstant %float -2
%_ptr_Output_v4float = OpTypePointer Output %v4float
%_ptr_Output_v2float = OpTypePointer Output %v2float
         %xy = OpVariable %_ptr_Output_v2float Output
    %float_2 = OpConstant %float 2
       %main = OpFunction %void None %3
          %5 = OpLabel
   %vertices = OpVariable %_ptr_Function__arr_v2float_uint_6 Function
               OpStore %vertices %19
         %36 = OpLoad %int %gl_ViewIndex
         %38 = OpAccessChain %_ptr_Uniform_mat4v4float %__0 %int_0 %36
         %39 = OpLoad %mat4v4float %38
         %41 = OpLoad %int %gl_VertexIndex
         %43 = OpAccessChain %_ptr_Function_v2float %vertices %41
         %44 = OpLoad %v2float %43
         %46 = OpVectorTimesScalar %v2float %44 %float_0_5
         %48 = OpCompositeExtract %float %46 0
         %49 = OpCompositeExtract %float %46 1
         %50 = OpCompositeConstruct %v4float %48 %49 %float_n2 %float_1
         %51 = OpMatrixTimesVector %v4float %39 %50
         %53 = OpAccessChain %_ptr_Output_v4float %_ %int_0
               OpStore %53 %51
         %56 = OpLoad %int %gl_VertexIndex
         %57 = OpAccessChain %_ptr_Function_v2float %vertices %56
         %58 = OpLoad %v2float %57
         %59 = OpCompositeConstruct %v2float %float_1 %float_1
         %60 = OpFAdd %v2float %58 %59
         %62 = OpCompositeConstruct %v2float %float_2 %float_2
         %63 = OpFDiv %v2float %60 %62
               OpStore %xy %63
               OpReturn
               OpFunctionEnd
